use crate::store::DataStore;
use async_trait::async_trait;
use dataloader::cached::Loader;
use dataloader::BatchFn;
use juniper::FieldError;
use std::collections::HashMap;
use std::fmt;

use crate::data::{Link, ResourceInfo, Tag};

#[derive(Debug, Clone)]
pub enum Error {
    SQLError(String),
}
impl From<FieldError> for Error {
    fn from(error: FieldError) -> Self {
        Error::SQLError(format!("{:}", error.message()))
    }
}
impl fmt::Display for Error {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Error::SQLError(error) => error.fmt(f),
        }
    }
}

#[derive(Clone)]
pub struct DataLoader {
    pub store: DataStore,
    pub links_by_target: Loader<String, Vec<Link>, Error, LinksByTarget>,
    pub links_by_referrer: Loader<String, Vec<Link>, Error, LinksByReferrer>,
    pub resource_info: Loader<String, ResourceInfo, Error, ResourceInfoByURL>,
    pub tags_by_target: Loader<String, Vec<Tag>, Error, TagsByTarget>,
    pub tags_by_name: Loader<String, Vec<Tag>, Error, TagsByName>,
}
impl fmt::Debug for DataLoader {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_struct("DataLoader")
            .field("store", &self.store)
            .finish()
    }
}

impl DataLoader {
    pub fn new(store: DataStore) -> DataLoader {
        DataLoader {
            store: store.clone(),
            links_by_target: Loader::new(LinksByTarget {
                store: store.clone(),
            }),
            links_by_referrer: Loader::new(LinksByReferrer {
                store: store.clone(),
            }),
            resource_info: Loader::new(ResourceInfoByURL {
                store: store.clone(),
            }),
            tags_by_target: Loader::new(TagsByTarget {
                store: store.clone(),
            }),
            tags_by_name: Loader::new(TagsByName {
                store: store.clone(),
            }),
        }
    }
}

pub struct LinksByReferrer {
    store: DataStore,
}

#[async_trait]
impl BatchFn<String, Vec<Link>> for LinksByReferrer {
    type Error = Error;
    async fn load(&self, urls: &[String]) -> HashMap<String, Result<Vec<Link>, Self::Error>> {
        urls.iter()
            .map(|url| {
                (
                    url.clone(),
                    self.store
                        .select_links_by_referrer(url)
                        .map_err(Error::from),
                )
            })
            .collect()
    }
}

pub struct LinksByTarget {
    store: DataStore,
}

#[async_trait]
impl BatchFn<String, Vec<Link>> for LinksByTarget {
    type Error = Error;
    async fn load(&self, urls: &[String]) -> HashMap<String, Result<Vec<Link>, Self::Error>> {
        urls.iter()
            .map(|url| {
                (
                    url.clone(),
                    self.store.select_links_by_target(url).map_err(Error::from),
                )
            })
            .collect()
    }
}

pub struct TagsByTarget {
    store: DataStore,
}

#[async_trait]
impl BatchFn<String, Vec<Tag>> for TagsByTarget {
    type Error = Error;
    async fn load(&self, urls: &[String]) -> HashMap<String, Result<Vec<Tag>, Self::Error>> {
        urls.iter()
            .map(|url| {
                (
                    url.clone(),
                    self.store.select_tags_by_target(url).map_err(Error::from),
                )
            })
            .collect()
    }
}

pub struct TagsByName {
    store: DataStore,
}

#[async_trait]
impl BatchFn<String, Vec<Tag>> for TagsByName {
    type Error = Error;
    async fn load(&self, names: &[String]) -> HashMap<String, Result<Vec<Tag>, Self::Error>> {
        names
            .iter()
            .map(|name| {
                (
                    name.clone(),
                    self.store.select_tags_by_name(name).map_err(Error::from),
                )
            })
            .collect()
    }
}

pub struct ResourceInfoByURL {
    store: DataStore,
}

#[async_trait]
impl BatchFn<String, ResourceInfo> for ResourceInfoByURL {
    type Error = Error;
    async fn load(&self, urls: &[String]) -> HashMap<String, Result<ResourceInfo, Self::Error>> {
        urls.iter()
            .map(|url| {
                (
                    url.clone(),
                    self.store.select_resource_by_url(url).map_err(Error::from),
                )
            })
            .collect()
    }
}
