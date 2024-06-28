use crate::{
    error::Error,
    fetch::{date_iter, locations_page, make_client, menus_on_date},
    parse::Locations,
    transpose::transposed,
};
use chrono::{DateTime, Utc};
use firestore::FirestoreDb;
use futures::{stream::FuturesUnordered, StreamExt};
use log::info;
use tokio::io::AsyncReadExt;
const CACHES_COLLECTION: &str = "caches";
#[derive(Debug)]
pub struct MenuCache<'a> {
    cached_at: DateTime<Utc>,
    locations: Locations<'a>,
}
#[derive(serde::Serialize, serde::Deserialize, Default)]
struct GCloudMenuCache {
    cached_at: DateTime<Utc>,
    data: Vec<u8>,
}

pub static REFRESH_INTERVAL: chrono::Duration = chrono::Duration::minutes(15);

impl<'a> MenuCache<'a> {
    async fn from_async(cache: GCloudMenuCache) -> Self {
        if cache.data.is_empty() {
            return MenuCache {
                cached_at: cache.cached_at,
                locations: Locations::default(),
            };
        }
        let mut decompress =
            async_compression::tokio::bufread::GzipDecoder::new(cache.data.as_slice());
        info!("Size of data compressed: {}", cache.data.len());
        let mut dst = String::with_capacity(cache.data.len() * 8);
        let _len = decompress
            .read_to_string(&mut dst)
            .await
            .expect("should succeed");
        info!("Size of data uncompressed: {}", dst.len());
        let locations: Locations =
            serde_json::from_str(&dst).expect("Data parse should always be valid");
        MenuCache {
            cached_at: cache.cached_at,
            locations,
        }
    }
}

impl<'a> Default for MenuCache<'a> {
    fn default() -> Self {
        Self {
            cached_at: Utc::now(),
            locations: Locations::default(),
        }
    }
}
impl<'a> MenuCache<'a> {
    pub async fn open() -> Result<Self, Error> {
        let cache = Self::fetch_from_db().await?;
        Ok(cache)
    }

    pub async fn maybe_refresh(&mut self) -> Result<bool, Error> {
        if self.get_time_since_refresh() > chrono::Duration::minutes(15) {
            self.refresh().await?;
            Ok(true)
        } else {
            Ok(false)
        }
    }

    pub fn get_time_since_refresh(&self) -> chrono::Duration {
        Utc::now().signed_duration_since(self.cached_at)
    }

    pub fn get_time_until_refresh(&self) -> chrono::Duration {
        REFRESH_INTERVAL - self.get_time_since_refresh()
    }

    async fn fetch_from_db() -> Result<Self, crate::error::Error> {
        let db = FirestoreDb::new("ucsc-menu").await?;
        let cache: GCloudMenuCache = db
            .fluent()
            .select()
            .by_id_in(CACHES_COLLECTION)
            .obj()
            .one("menu")
            .await?
            .unwrap_or_default(); // default is an empty cache
        Ok(MenuCache::from_async(cache).await)
    }

    async fn to_db_representation(&self) -> GCloudMenuCache {
        let json = serde_json::to_string(self.locations()).unwrap();
        let mut compressed = Vec::with_capacity(json.len() / 4);
        let mut compress =
            async_compression::tokio::bufread::GzipEncoder::new(std::io::Cursor::new(json));
        compress
            .read_buf(&mut compressed)
            .await
            .expect("This should succeed");
        GCloudMenuCache {
            cached_at: self.cached_at,
            data: compressed,
        }
    }

    async fn save_to_db(&self) -> Result<(), firestore::errors::FirestoreError> {
        let cache: GCloudMenuCache = self.to_db_representation().await;
        let db = FirestoreDb::new("ucsc-menu").await?;
        db.fluent()
            .update()
            .in_col(CACHES_COLLECTION)
            .document_id("menu")
            .object(&cache)
            .execute()
            .await?;
        Ok(())
    }
    /// Returns whether or not it refreshed. Will return error if it fails
    async fn refresh(&mut self) -> Result<(), crate::error::Error> {
        let client = make_client();
        let locations_page = locations_page(&client).await?;
        let mut locations = {
            let parsed = scraper::Html::parse_document(&locations_page);
            let locations: Locations = Locations::from_html_element(parsed.root_element())?;
            locations
        };
        {
            let start_date = chrono::Utc::now().date_naive() - chrono::Duration::days(1); // subtract one day to make sure we try to get today's menu due to timezones
            let week_menus: FuturesUnordered<_> = date_iter(start_date, 10)
                .map(|x| menus_on_date(&client, &locations, Some(x)))
                .collect();
            let week_menus: Vec<_> = week_menus.collect().await;
            let valid_week_menus = week_menus.into_iter().filter_map(Result::ok).collect();

            let valid_week_menus: Vec<_> = transposed(valid_week_menus)
                .into_iter()
                .map(|v| -> Vec<_> {
                    v.into_iter()
                        .map(|s| scraper::Html::parse_document(&s))
                        .collect()
                })
                .collect();
            let parsed_week_menus_iter = valid_week_menus.iter();
            for (location, htmls) in locations.iter_mut().zip(parsed_week_menus_iter) {
                location.add_meals(htmls.iter())?;
            }
            self.locations =
                serde_json::from_str(&serde_json::to_string(&locations).unwrap()).unwrap();
        };
        self.cached_at = Utc::now();
        self.save_to_db().await?;
        Ok(())
    }

    pub const fn locations(&self) -> &Locations<'a> {
        &self.locations
    }
}

#[cfg(test)]
mod tests {

    use std::time::Instant;

    use super::*;

    #[tokio::test]
    async fn test_open() {
        pretty_env_logger::init();
        let _mc = MenuCache::open().await.unwrap();
    }

    #[tokio::test]
    async fn test_refresh() {
        let mut mc = MenuCache::open().await.unwrap();
        let start = Instant::now();
        mc.refresh().await.unwrap();
        println!("{:?}", start.elapsed());
    }
}
