use std::path::{Path, PathBuf};
use tokio::fs;

use super::MenuCache;

#[derive(Debug)]
pub struct FileStore(PathBuf);

impl FileStore {
    pub async fn open(p: impl AsRef<Path>) -> crate::Result<Self> {
        let p = p.as_ref();
        // fs::File::options()
        // .create(true)
        // .write(true)
        // .read(true)
        // .open(p)
        // .await?;

        Ok(Self(p.to_owned()))
    }

    pub async fn load(&self) -> crate::Result<Option<MenuCache>> {
        if fs::try_exists(&self.0).await? {
            let f = fs::File::open(&self.0).await?;
            let mut f = f.into_std().await;
            serde_json::from_reader(&mut f).map_err(From::from)
        } else {
            Ok(None)
        }
    }

    pub async fn save(&self, value: &MenuCache) -> crate::Result<()> {
        let f = fs::File::options()
            .create(true)
            .write(true)
            .open(&self.0)
            .await?;
        // TODO
        let mut f = f.into_std().await;
        serde_json::to_writer_pretty(&mut f, value).map_err(From::from)
    }
}
