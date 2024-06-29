mod firestore;
mod local;
mod multithreaded_cache;

use std::path::Path;

use chrono::{DateTime, Duration, Utc};
use serde::{Deserialize, Serialize};

use crate::{fetch, parse::Locations};

use firestore::Firestore;
use local::FileStore;
pub use multithreaded_cache::MultithreadedCache as Multithreaded;

pub const REFRESH_INTERVAL: Duration = Duration::minutes(15);

#[derive(Debug, Serialize, Deserialize)]
pub struct MenuCache {
    cached_at: DateTime<Utc>,
    locations: Locations,
}

impl Default for MenuCache {
    fn default() -> Self {
        Self {
            cached_at: Utc::now(),
            locations: Locations::default(),
        }
    }
}

impl MenuCache {
    #[inline]
    #[must_use]
    pub fn time_since_refresh(&self) -> chrono::Duration {
        Utc::now().signed_duration_since(self.cached_at)
    }

    #[inline]
    #[must_use]
    pub fn time_until_refresh(&self) -> chrono::Duration {
        REFRESH_INTERVAL - self.time_since_refresh()
    }

    #[inline]
    #[must_use]
    pub fn needs_refresh(&self) -> bool {
        self.time_since_refresh() > REFRESH_INTERVAL
    }

    #[inline]
    #[must_use]
    pub const fn locations(&self) -> &Locations {
        &self.locations
    }

    pub async fn load() -> crate::Result<Self> {
        let client = fetch::make_client();
        let locations = Locations::load(&client).await?;
        Ok(Self {
            locations,
            cached_at: Utc::now(),
        })
    }
}

#[derive(Debug)]
#[non_exhaustive]
pub enum Store {
    Cloud(Firestore),
    Local(FileStore),
    AdHoc,
}

impl Store {
    #[inline]
    pub async fn cloud() -> crate::Result<Self> {
        Firestore::open().await.map(Self::Cloud)
    }

    #[inline]
    pub async fn local(p: impl AsRef<Path>) -> crate::Result<Self> {
        FileStore::open(p).await.map(Self::Local)
    }

    pub async fn load(&mut self) -> crate::Result<MenuCache> {
        let value = match self {
            Self::Cloud(fs) => fs.load().await?,
            Self::Local(f) => f.load().await?,
            Self::AdHoc => None,
        };

        match value {
            Some(v) => Ok(v),
            None => {
                let v = MenuCache::load().await?;
                self.save(&v).await?;
                Ok(v)
            }
        }
    }

    pub async fn save(&mut self, data: &MenuCache) -> crate::Result<()> {
        match self {
            Self::Cloud(fs) => fs.save(data).await,
            Self::Local(f) => f.save(data).await,
            Self::AdHoc => Ok(()),
        }
    }

    pub async fn refresh(&mut self, data: &mut MenuCache) -> crate::Result<()> {
        *data = MenuCache::load().await?;
        self.save(data).await
    }

    pub async fn maybe_refresh(&mut self, data: &mut MenuCache) -> crate::Result<bool> {
        if data.needs_refresh() {
            self.refresh(data).await?;
            Ok(true)
        } else {
            Ok(false)
        }
    }
}
