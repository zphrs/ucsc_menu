use crate::parse::Locations;
use async_compression::tokio::bufread;
use chrono::{DateTime, Utc};
use firestore::FirestoreDb;
use log::info;
use tokio::io::AsyncReadExt;

use super::MenuCache;

const CACHES_COLLECTION: &str = "caches";

#[derive(Debug)]
pub struct Firestore {
    db: FirestoreDb,
    // data: String,
}

impl Firestore {
    pub async fn open() -> crate::Result<Self> {
        let db = FirestoreDb::new("ucsc-menu").await?;
        Ok(Self {
            db,
            // data: String::new(),
        })
    }

    pub async fn load(&mut self) -> crate::Result<Option<MenuCache>> {
        if let Some(cache) = self
            .db
            .fluent()
            .select()
            .by_id_in(CACHES_COLLECTION)
            .obj::<GCloudMenuCache>()
            .one("menu")
            .await?
        {
            Ok(Some(cache.decompress().await))
        } else {
            Ok(None)
        }
        // Ok(cache.decompress(&mut self.data).await)
    }

    pub async fn save(&self, cache: &MenuCache) -> crate::Result<()> {
        let compressed = GCloudMenuCache::compress(cache).await;
        self.db
            .fluent()
            .update()
            .in_col(CACHES_COLLECTION)
            .document_id("menu")
            .object(&compressed)
            // need to specify type because of dependency_on_unit_never_type_fallback
            .execute::<()>()
            .await?;
        Ok(())
    }
}

#[derive(serde::Serialize, serde::Deserialize, Default)]
struct GCloudMenuCache {
    cached_at: DateTime<Utc>,
    data: Vec<u8>,
}

impl GCloudMenuCache {
    async fn decompress<'a>(self /* dst: &mut String */) -> MenuCache {
        if self.data.is_empty() {
            return MenuCache {
                cached_at: self.cached_at,
                locations: Locations::default(),
            };
        }
        let mut decompress = bufread::GzipDecoder::new(self.data.as_slice());
        info!("Size of data compressed: {}", self.data.len());
        let mut dst = String::new();
        dst.reserve(self.data.len() * 8);
        let _len = decompress
            .read_to_string(&mut dst)
            .await
            .expect("should succeed");
        info!("Size of data uncompressed: {}", dst.len());
        let locations: Locations =
            serde_json::from_str(&*dst).expect("Data parse should always be valid");
        MenuCache {
            cached_at: self.cached_at,
            locations,
        }
    }

    // TODO: better name
    async fn compress(data: &MenuCache) -> Self {
        let json = serde_json::to_string(data.locations()).unwrap();
        let mut compressed = Vec::with_capacity(json.len() / 4);
        let mut compress = bufread::GzipEncoder::new(std::io::Cursor::new(json));
        compress
            .read_buf(&mut compressed)
            .await
            .expect("This should succeed");
        Self {
            cached_at: data.cached_at,
            data: compressed,
        }
    }
}

#[cfg(test)]
mod tests {

    use std::time::Instant;

    use super::*;

    #[tokio::test]
    async fn test_open() {
        pretty_env_logger::init();
        let mut db = Firestore::open().await.unwrap();
        let _mc = db.load().await.unwrap();
    }

    #[tokio::test]
    async fn test_refresh() {
        let mut db = Firestore::open().await.unwrap();
        let mut mc = db.load().await.unwrap();
        let start = Instant::now();
        todo!();
        println!("{:?}", start.elapsed());
    }
}
