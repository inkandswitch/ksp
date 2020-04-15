use crate::data::{
    InputLink, InputResource, InputTag, Link, LinkKind, Resource, ResourceInfo, Tag,
};
use async_trait::async_trait;
use dataloader::cached::Loader;
use dataloader::BatchFn;
use dirs;
pub use juniper::{FieldError, FieldResult};
use log;
use r2d2_sqlite::SqliteConnectionManager;
use rusqlite::{named_params, Connection, Row};
use std::collections::HashMap;
use std::fmt;
use std::{include_str, io};

pub type DecodeResult<T> = Result<T, FieldError>;

trait RowDecoder
where
    Self: std::marker::Sized,
{
    fn decode_row(row: &Row<'_>) -> Result<Self, rusqlite::Error>;
    fn decode_rows(rows: &mut rusqlite::Rows) -> Result<Vec<Self>, FieldError> {
        let mut records = Vec::new();
        while let Some(row) = rows.next()? {
            records.push(Self::decode_row(&row)?);
        }
        Ok(records)
    }
}

type Pool = r2d2::Pool<SqliteConnectionManager>;

pub struct DataStore {
    pool: r2d2::Pool<SqliteConnectionManager>,

    links_by_referrer: Loader<String, Vec<Link>, Error, LinksByReferrer>,
    links_by_target: Loader<String, Vec<Link>, Error, LinksByTarget>,
    resource_info_by_url: Loader<String, ResourceInfo, Error, ResourceInfoByURL>,
    tags_by_target: Loader<String, Vec<Tag>, Error, TagsByTarget>,
    tags_by_name: Loader<String, Vec<Tag>, Error, TagsByName>,
}

impl DataStore {
    pub fn new(pool: Pool) -> Self {
        DataStore {
            links_by_target: Loader::new(LinksByTarget::new(&pool)),
            links_by_referrer: Loader::new(LinksByReferrer::new(&pool)),
            tags_by_target: Loader::new(TagsByTarget::new(&pool)),
            tags_by_name: Loader::new(TagsByName::new(&pool)),
            resource_info_by_url: Loader::new(ResourceInfoByURL::new(&pool)),
            pool: pool,
        }
    }
    pub fn open() -> io::Result<Self> {
        let mut path = dirs::home_dir().expect("Unable to locate user home directory");
        path.push(".knowledge-service");
        // Ensure that there is such directory
        std::fs::create_dir_all(&path)?;
        path.push("knowledge.sqlite");
        let manager = SqliteConnectionManager::file(&path).with_init(DataStore::create_tables);
        let pool = r2d2::Pool::new(manager).expect("Failed to initialize connection manager");
        let store = DataStore::new(pool);
        println!("Data base was initialized at {:?}", path.to_str());
        Ok(store)
    }

    pub(crate) fn create_tables(connection: &mut Connection) -> Result<(), rusqlite::Error> {
        Ok(connection.execute_batch(include_str!("../sql/create_tables.sql"))?)
    }
    pub(crate) fn insert_resource(&self, input: &InputResource) -> DecodeResult<Resource> {
        let connection = self.pool.get()?;
        let mut insert = connection.prepare_cached(include_str!("../sql/insert_resource.sql"))?;
        insert.execute_named(named_params! {
          ":url": input.url,
          ":title": input.title,
          ":description": input.description,
          ":cid": input.cid
        })?;

        Ok(Resource::from(input))
    }
    pub(crate) fn insert_links(
        &self,
        referrer_url: &str,
        links: &Vec<InputLink>,
    ) -> FieldResult<()> {
        log::info!("Inserting {:} resource links into db", links.len());
        let connection = self.pool.get()?;
        let mut insert_inline =
            connection.prepare_cached(include_str!("../sql/insert_inline_link.sql"))?;
        let mut insert_reference =
            connection.prepare_cached(include_str!("../sql/insert_reference_link.sql"))?;

        for link in links {
            match link.kind {
                LinkKind::Inline => {
                    insert_inline.execute_named(named_params! {
                        ":referrer_url": referrer_url,
                        ":referrer_fragment": link.referrer_fragment,
                        ":referrer_location": link.referrer_location,
                        ":target_url": link.target_url,
                        ":name": link.name,
                        ":title": link.title
                    })?;
                    log::info!("Link {:} -> {:}resource", referrer_url, link.target_url);
                }
                LinkKind::Reference => {
                    insert_reference.execute_named(named_params! {
                      ":referrer_url": referrer_url,
                        ":referrer_fragment": link.referrer_fragment,
                        ":referrer_location": link.referrer_location,
                        ":target_url": link.target_url,
                        ":identifier": match &link.identifier {
                            Some(name) => name,
                            None => "",
                        },
                        ":name": link.name,
                        ":title": link.title
                    })?;
                    log::info!("Link {:} -> {:} resource", referrer_url, link.target_url);
                }
            }
        }

        Ok(())
    }
    pub(crate) fn insert_tags(&self, target_url: &str, tags: &Vec<InputTag>) -> DecodeResult<()> {
        log::info!("Inserting {:} resource tags into db", tags.len());
        let connection = self.pool.get()?;
        let mut insert = connection.prepare_cached(include_str!("../sql/insert_tag.sql"))?;
        let no_fragment = String::new();
        for tag in tags {
            insert.execute_named(named_params! {
              ":name": tag.name,
              ":target_url": target_url,
              ":target_fragment": tag.target_fragment.as_ref().unwrap_or(&no_fragment),
              ":target_location": tag.target_location,
            })?;
            log::info!("Add #{:} tag to {:}", tag.name, target_url);
        }
        Ok(())
    }

    pub(crate) async fn find_resource_by_url(&self, url: &str) -> DecodeResult<ResourceInfo> {
        self.resource_info_by_url
            .load(url.to_string())
            .await
            .map_err(FieldError::from)
    }

    pub(crate) async fn find_links_by_target(&self, url: &str) -> FieldResult<Vec<Link>> {
        self.links_by_target
            .load(url.to_string())
            .await
            .map_err(FieldError::from)
    }

    pub(crate) async fn find_links_by_referrer(&self, url: &str) -> FieldResult<Vec<Link>> {
        self.links_by_referrer
            .load(url.to_string())
            .await
            .map_err(FieldError::from)
    }

    pub(crate) async fn find_tags_by_target(&self, target_url: &str) -> DecodeResult<Vec<Tag>> {
        self.tags_by_target
            .load(target_url.to_string())
            .await
            .map_err(FieldError::from)
    }
    pub(crate) async fn find_tags_by_name(&self, name: &str) -> DecodeResult<Vec<Tag>> {
        self.tags_by_name
            .load(name.to_string())
            .await
            .map_err(FieldError::from)
    }
}

impl Clone for DataStore {
    fn clone(&self) -> Self {
        DataStore::new(self.pool.clone())
    }
}

impl fmt::Debug for DataStore {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_struct("DataLoader")
            .field("pool", &self.pool)
            .finish()
    }
}

impl RowDecoder for Link {
    fn decode_row(row: &rusqlite::Row<'_>) -> Result<Self, rusqlite::Error> {
        Ok(Link {
            kind: match row.get(0)? {
                0 => LinkKind::Inline,
                1 => LinkKind::Reference,
                _ => LinkKind::Inline,
            },
            referrer_url: row.get(1)?,
            referrer_cid: row.get(2)?,
            referrer_title: row.get(3)?,
            referrer_description: row.get(4)?,
            referrer_fragment: row.get(5)?,
            referrer_location: row.get(6)?,

            target_url: row.get(7)?,
            identifier: row.get(8)?,
            name: row.get(9)?,
            title: row.get(10)?,
        })
    }
}

impl RowDecoder for Tag {
    fn decode_row(row: &Row<'_>) -> Result<Self, rusqlite::Error> {
        Ok(Tag {
            target_url: row.get(0)?,
            name: row.get(1)?,
            target_fragment: row.get(2)?,
            target_location: row.get(3)?,
        })
    }
}

impl RowDecoder for ResourceInfo {
    fn decode_row(row: &Row) -> Result<Self, rusqlite::Error> {
        Ok(ResourceInfo {
            cid: row.get(0)?,
            title: row.get(1)?,
            description: row.get(2)?,
        })
    }
}

pub struct LinksByTarget {
    pool: Pool,
}
impl LinksByTarget {
    pub fn new(pool: &Pool) -> Self {
        LinksByTarget { pool: pool.clone() }
    }
    pub fn select(&self, target_url: &str) -> Result<Vec<Link>, FieldError> {
        log::info!("selecting links by target {:} in db", target_url);
        let connection = self.pool.get()?;

        let mut select =
            connection.prepare_cached(include_str!("../sql/select_links_by_target.sql"))?;
        let mut rows = select.query_named(named_params! {":target_url": target_url})?;
        Link::decode_rows(&mut rows)
    }
}

#[async_trait]
impl BatchFn<String, Vec<Link>> for LinksByTarget {
    type Error = Error;
    async fn load(&self, urls: &[String]) -> HashMap<String, Result<Vec<Link>, Self::Error>> {
        urls.iter()
            .map(|url| (url.clone(), self.select(url).map_err(Error::from)))
            .collect()
    }
}

pub struct LinksByReferrer {
    pool: Pool,
}
impl LinksByReferrer {
    pub fn new(pool: &Pool) -> Self {
        LinksByReferrer { pool: pool.clone() }
    }
    pub fn select(&self, referrer_url: &str) -> Result<Vec<Link>, FieldError> {
        log::info!("selecting links by referrer {:} in db", referrer_url);
        let connection = self.pool.get()?;
        let mut select =
            connection.prepare_cached(include_str!("../sql/select_links_by_referrer.sql"))?;
        let mut rows = select.query_named(named_params! {":referrer_url": referrer_url})?;
        Link::decode_rows(&mut rows)
    }
}
#[async_trait]
impl BatchFn<String, Vec<Link>> for LinksByReferrer {
    type Error = Error;
    async fn load(&self, urls: &[String]) -> HashMap<String, Result<Vec<Link>, Self::Error>> {
        urls.iter()
            .map(|url| (url.to_string(), self.select(&url).map_err(Error::from)))
            .collect()
    }
}

pub struct TagsByTarget {
    pool: Pool,
}

impl TagsByTarget {
    pub fn new(pool: &Pool) -> Self {
        TagsByTarget { pool: pool.clone() }
    }
    pub fn select(&self, target_url: &str) -> Result<Vec<Tag>, FieldError> {
        log::info!("selecting tags by target {:} in db", target_url);

        let connection = self.pool.get()?;

        let mut select =
            connection.prepare_cached(include_str!("../sql/select_tags_by_target.sql"))?;
        let mut rows = select.query_named(named_params! {":target_url": target_url})?;
        Tag::decode_rows(&mut rows)
    }
}
#[async_trait]
impl BatchFn<String, Vec<Tag>> for TagsByTarget {
    type Error = Error;
    async fn load(&self, urls: &[String]) -> HashMap<String, Result<Vec<Tag>, Self::Error>> {
        urls.iter()
            .map(|url| (url.clone(), self.select(&url).map_err(Error::from)))
            .collect()
    }
}

pub struct TagsByName {
    pool: Pool,
}

impl TagsByName {
    pub fn new(pool: &Pool) -> Self {
        TagsByName { pool: pool.clone() }
    }
    pub fn select(&self, name: &str) -> Result<Vec<Tag>, FieldError> {
        log::info!("selecting tags by name #{:} in db", name);

        let connection = self.pool.get()?;

        let mut select =
            connection.prepare_cached(include_str!("../sql/select_tags_by_name.sql"))?;
        let mut rows = select.query_named(named_params! {":name": name})?;
        Tag::decode_rows(&mut rows)
    }
}
#[async_trait]
impl BatchFn<String, Vec<Tag>> for TagsByName {
    type Error = Error;
    async fn load(&self, names: &[String]) -> HashMap<String, Result<Vec<Tag>, Self::Error>> {
        names
            .iter()
            .map(|name| (name.clone(), self.select(&name).map_err(Error::from)))
            .collect()
    }
}
pub struct ResourceInfoByURL {
    pool: Pool,
}
impl ResourceInfoByURL {
    pub fn new(pool: &Pool) -> Self {
        ResourceInfoByURL { pool: pool.clone() }
    }
    pub fn select(&self, url: &str) -> Result<ResourceInfo, FieldError> {
        log::info!("selecting a resource in db{:}", url);
        let connection = self.pool.get()?;
        let mut select =
            connection.prepare_cached(include_str!("../sql/select_resource_by_url.sql"))?;
        let info = select.query_row_named(named_params! {":url": url}, ResourceInfo::decode_row)?;

        Ok(info)
    }
}

#[async_trait]
impl BatchFn<String, ResourceInfo> for ResourceInfoByURL {
    type Error = Error;
    async fn load(&self, urls: &[String]) -> HashMap<String, Result<ResourceInfo, Self::Error>> {
        urls.iter()
            .map(|url| (url.to_string(), self.select(url).map_err(Error::from)))
            .collect()
    }
}

#[derive(Debug, Clone)]
pub enum Error {
    SQLError(String),
}
impl From<FieldError> for Error {
    fn from(error: FieldError) -> Self {
        Error::SQLError(format!("{:}", error.message()))
    }
}
impl From<rusqlite::Error> for Error {
    fn from(error: rusqlite::Error) -> Self {
        Error::SQLError(format!("{:}", error))
    }
}
impl fmt::Display for Error {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Error::SQLError(error) => error.fmt(f),
        }
    }
}
