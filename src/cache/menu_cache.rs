use std::{pin::Pin, sync::OnceLock};

use crate::{
    error::Error,
    fetch::{date_iter, fetch_locations_page, fetch_menus_on_date, make_client},
    parse::Locations,
    transpose::transposed,
};
use chrono::{DateTime, NaiveDate, NaiveDateTime, Offset, TimeDelta, Utc};
use firestore::{FirestoreDb, FirestoreQueryCollection};
use futures::{stream::FuturesUnordered, StreamExt};
use scraper::Html;

const CACHES_COLLECTION: &str = "caches";

#[derive(Debug)]
pub struct MenuCache<'a> {
    cached_at: DateTime<Utc>,
    locations: Locations<'a>,
}

#[derive(serde::Serialize, serde::Deserialize, Default)]
struct InDbMenuCache {
    cached_at: DateTime<Utc>,
    data: String,
}

impl InDbMenuCache {
    pub fn new(data: String) -> Self {
        Self {
            data,
            ..Default::default()
        }
    }
}

impl<'a> From<InDbMenuCache> for MenuCache<'a> {
    fn from(cache: InDbMenuCache) -> Self {
        if cache.data.is_empty() {
            return MenuCache {
                cached_at: cache.cached_at,
                locations: Locations::default(),
            };
        }
        let locations: Locations =
            serde_json::from_str(&cache.data).expect("Data parse should always be valid");
        MenuCache {
            cached_at: cache.cached_at,
            locations,
        }
    }
}

impl<'a> From<MenuCache<'a>> for InDbMenuCache {
    fn from(cache: MenuCache<'a>) -> Self {
        InDbMenuCache {
            cached_at: cache.cached_at,
            data: serde_json::to_string(&cache.locations).unwrap(),
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

static DB: OnceLock<FirestoreDb> = OnceLock::new();

impl<'a> MenuCache<'a> {
    pub fn is_empty(&self) -> bool {
        self.locations.is_empty()
    }
    pub async fn open() -> Result<Self, Error> {
        let cache = Self::fetch_from_db().await?;
        Ok(cache)
    }

    pub async fn maybe_refresh(&mut self) -> Result<bool, Error> {
        if Utc::now().signed_duration_since(self.cached_at) > chrono::Duration::minutes(1) {
            self.refresh().await?;
            Ok(true)
        } else {
            Ok(false)
        }
    }

    async fn fetch_from_db() -> Result<Self, crate::error::Error> {
        let db = FirestoreDb::new("ucsc-menu").await?;
        let cache: InDbMenuCache = db
            .fluent()
            .select()
            .by_id_in(CACHES_COLLECTION)
            .obj()
            .one("menu")
            .await?
            .unwrap_or_default(); // default is an empty cache
        Ok(cache.into())
    }

    fn to_db_representation(&self) -> InDbMenuCache {
        InDbMenuCache {
            data: serde_json::to_string(&self.locations).unwrap(),
            cached_at: self.cached_at,
        }
    }

    async fn save_to_db(&self) -> Result<(), firestore::errors::FirestoreError> {
        let cache: InDbMenuCache = self.to_db_representation();
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
        let locations_page = fetch_locations_page(&client).await?;
        let mut locations = {
            let parsed = scraper::Html::parse_document(&locations_page);
            let locations: Locations = Locations::from_html_element(parsed.root_element())?;
            locations
        };
        {
            let start_date = chrono::Utc::now().date_naive() - chrono::Duration::days(1); // subtract one day to make sure we try to get today's menu due to timezones
            let week_menus: FuturesUnordered<_> = date_iter(start_date, 10)
                .map(|x| fetch_menus_on_date(&client, &locations, Some(x)))
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

    pub fn locations(&self) -> &Locations<'a> {
        &self.locations
    }
}

#[cfg(test)]
mod tests {
    use std::time::Instant;

    use super::*;

    #[tokio::test]
    async fn test_open() {
        let _mc = MenuCache::open().await.unwrap();
    }

    #[tokio::test]
    async fn test_refresh() {
        let mut mc = MenuCache::open().await.unwrap();
        mc.refresh().await.unwrap();
    }
}
