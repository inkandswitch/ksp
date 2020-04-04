use async_std::fs::File;
use async_std::io::{Error, ErrorKind, Read, ReadExt, Result, Seek};
pub use std::convert::TryFrom;
use std::fmt;
use std::path::Path;
use url::Url;

#[derive(Debug, Clone)]
pub enum Resource {
  File(Url),
}

impl Resource {
  pub fn url(&self) -> &Url {
    match self {
      Resource::File(url) => &url,
    }
  }

  pub fn from_file_path(path: &Path) -> Result<Resource> {
    Resource::try_from(path)
  }

  pub async fn reader(&self) -> Result<impl Read + ReadExt + Seek + Drop + fmt::Debug> {
    match self {
      // .unwrap is fine as we know it's file:// url
      Resource::File(url) => File::open(url.to_file_path().unwrap()).await,
    }
  }

  pub async fn read_to_string<'a>(&self, buf: &'a mut String) -> Result<usize> {
    let mut reader = self.reader().await?;
    reader.read_to_string(buf).await
  }
}

impl TryFrom<&Path> for Resource {
  type Error = Error;

  fn try_from(path: &Path) -> Result<Self> {
    let url = Url::from_file_path(&path)
      .map_err(|_| Error::new(ErrorKind::Other, "Unable to create file URL for path"))?;

    Ok(Resource::File(url))
  }
}
