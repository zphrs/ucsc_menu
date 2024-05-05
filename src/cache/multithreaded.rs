use super::menu_cache::{self, MenuCache};
use crate::{error::Error, parse::Locations};
use std::{
    borrow::BorrowMut,
    ops::{Deref, DerefMut},
    sync::Arc,
};

use futures::FutureExt;
use futures_locks::RwLock;
use juniper::{EmptyMutation, EmptySubscription, RootNode};

#[derive(Debug)]
pub struct MultithreadedCache<'a>(RwLock<MenuCache<'a>>);

impl<'a> MultithreadedCache<'a> {
    pub async fn new() -> Result<Self, crate::error::Error> {
        let menu = MenuCache::open().await?;

        Ok(Self(RwLock::new(menu)))
    }

    pub async fn refresh(&self) -> Result<bool, Error> {
        // spawn local thread to do the refreshing
        let mut new_menu = MenuCache::open().await?;
        let refreshed = new_menu.maybe_refresh().await?;
        if refreshed {
            let mut guard = self.0.write().await;
            *guard = new_menu;
        }

        Ok(refreshed)
    }

    pub async fn get<'b>(&'b self) -> impl Deref<Target = MenuCache<'a>> + 'b
    where
        'a: 'b,
    {
        self.0.read().await
    }
    pub async fn get_root_node<'b>(
        &'b self,
    ) -> RootNode<'static, Locations, EmptyMutation, EmptySubscription> {
        RootNode::new(
            self.0.read().await.locations().to_owned(),
            EmptyMutation::<()>::new(),
            EmptySubscription::<()>::new(),
        )
    }
}

#[cfg(test)]
mod tests {
    use std::thread;

    use futures::future::join_all;

    use super::*;

    #[tokio::test(flavor = "multi_thread")]
    async fn test_refresh() {
        let menu = MultithreadedCache::new().await.unwrap();
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
