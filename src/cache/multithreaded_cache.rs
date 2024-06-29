use super::{MenuCache, Store};
use crate::error::Error;
use std::ops::Deref;

use futures_locks::RwLock;

#[derive(Debug)]
pub struct MultithreadedCache(RwLock<MultithreadedInner>);

#[derive(Debug)]
pub struct MultithreadedInner {
    store: Store,
    data: MenuCache,
}

impl MultithreadedCache {
    #[must_use]
    pub async fn new(mut store: Store) -> crate::Result<Self> {
        let data = store.load().await?;
        Ok(Self(RwLock::new(MultithreadedInner { store, data })))
    }

    pub async fn refresh(&self) -> Result<bool, Error> {
        let needs_refresh = self.0.read().await.data.needs_refresh();

        if needs_refresh {
            // don't lock until we want to actually write to the db
            // that is, don't use Store::refresh
            let new_data = MenuCache::load().await?;
            let mut grd = self.0.write().await;
            grd.store.save(&new_data).await?;
            grd.data = new_data;
        }

        Ok(needs_refresh)
    }

    pub async fn get(&self) -> impl Deref<Target = MenuCache> + '_ {
        DataGuard(self.0.read().await)
    }
}

#[derive(Debug)]
struct DataGuard(futures_locks::RwLockReadGuard<MultithreadedInner>);

impl Deref for DataGuard {
    type Target = MenuCache;

    #[inline]
    fn deref(&self) -> &Self::Target {
        &self.0.data
    }
}

#[cfg(test)]
mod tests {

    use super::*;

    #[tokio::test(flavor = "multi_thread")]
    async fn test_refresh() {
        let menu = MultithreadedCache::new(Store::AdHoc).await.unwrap();
        menu.refresh().await.unwrap();
        // try having multiple threads read from menu at the same time
        // using the get() function
        tokio_scoped::scope(|s| {
            let mut scope = s;
            scope.spawn(async {
                // update the menu
                // (*menu.get_write()).maybe_refresh().await.unwrap();
                menu.refresh().await.unwrap();
            });
            for _ in 0..10 {
                scope = scope.spawn(async {
                    let lock = menu.get().await;
                    let locations = lock.locations();
                    //
                    println!("len of locations: {}", locations.iter().len());
                });
            }
        });
    }
}
