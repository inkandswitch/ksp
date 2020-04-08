use crate::data::{
    InputLink, InputResource, InputTag, Link, LinkKind, Resource, ResourceInfo, Tag,
};
use dirs;
use juniper::FieldResult;
use r2d2_sqlite::SqliteConnectionManager;
use rusqlite::{named_params, Connection, Row};
use std::{include_str, io};

pub type DecodeResult<T> = Result<T, rusqlite::Error>;

trait RowDecoder
where
    Self: std::marker::Sized,
{
    fn decode_row(row: &Row<'_>) -> DecodeResult<Self>;
    fn decode_rows(rows: &mut rusqlite::Rows) -> FieldResult<Vec<Self>> {
        let mut records = Vec::new();
        while let Some(row) = rows.next()? {
            records.push(Self::decode_row(&row)?);
        }
        Ok(records)
    }
}

pub struct DataStore {
    pool: r2d2::Pool<SqliteConnectionManager>,
}

impl RowDecoder for Link {
    fn decode_row(row: &rusqlite::Row<'_>) -> DecodeResult<Self> {
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
    fn decode_row(row: &Row<'_>) -> DecodeResult<Self> {
        Ok(Tag {
            target_url: row.get(0)?,
            name: row.get(1)?,
            target_fragment: row.get(2)?,
            target_location: row.get(3)?,
        })
    }
}

impl RowDecoder for ResourceInfo {
    fn decode_row(row: &Row) -> DecodeResult<Self> {
        Ok(ResourceInfo {
            cid: row.get(0)?,
            title: row.get(1)?,
            description: row.get(2)?,
        })
    }
}

impl DataStore {
    pub fn open() -> io::Result<Self> {
        let mut path = dirs::home_dir().expect("Unable to locate user home directory");
        path.push(".knowledge-service");
        // Ensure that there is such directory
        std::fs::create_dir_all(&path)?;
        path.push("knowledge.sqlite");
        let manager = SqliteConnectionManager::file(&path).with_init(DataStore::create_tables);
        let pool = r2d2::Pool::new(manager).expect("Failed to initialize connection manager");
        let store = DataStore { pool: pool };
        println!("Data base was initialized at {:?}", path.to_str());
        Ok(store)
    }
    pub(crate) fn create_tables(connection: &mut Connection) -> Result<(), rusqlite::Error> {
        connection.execute_batch(include_str!("../sql/create_tables.sql"))
    }
    pub(crate) fn insert_resource(&self, input: &InputResource) -> FieldResult<Resource> {
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
        let connection = self.pool.get()?;
        for link in links {
            match link.kind {
                LinkKind::Inline => {
                    let mut insert =
                        connection.prepare_cached(include_str!("../sql/insert_inline_link.sql"))?;
                    insert.execute_named(named_params! {
                        ":referrer_url": referrer_url,
                        ":referrer_fragment": link.referrer_fragment,
                        ":referrer_location": link.referrer_location,
                        ":target_url": link.target_url,
                        ":name": link.name,
                        ":title": link.title
                    })?;
                }
                LinkKind::Reference => {
                    let mut insert = connection
                        .prepare_cached(include_str!("../sql/insert_reference_link.sql"))?;

                    insert.execute_named(named_params! {
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
                }
            }
        }
        Ok(())
    }
    pub(crate) fn insert_tags(&self, target_url: &str, tags: &Vec<InputTag>) -> FieldResult<()> {
        let connection = self.pool.get()?;
        for tag in tags {
            let mut insert = connection.prepare_cached(include_str!("../sql/insert_tag.sql"))?;
            insert.execute_named(named_params! {
              ":name": tag.name,
              ":target_url": target_url,
              ":target_fragment": tag.target_fragment,
              ":target_location": tag.target_location,
            })?;
        }
        Ok(())
    }

    pub(crate) fn select_resource_by_url(&self, url: &str) -> FieldResult<ResourceInfo> {
        let connection = self.pool.get()?;
        let mut select =
            connection.prepare_cached(include_str!("../sql/select_resource_by_url.sql"))?;
        let info = select.query_row_named(named_params! {":url": url}, ResourceInfo::decode_row)?;

        Ok(info)
    }

    pub(crate) fn select_links_by_referrer(&self, referrer_url: &str) -> FieldResult<Vec<Link>> {
        let connection = self.pool.get()?;
        let mut select =
            connection.prepare_cached(include_str!("../sql/select_links_by_referrer.sql"))?;
        let mut rows = select.query_named(named_params! {":referrer_url": referrer_url})?;
        Link::decode_rows(&mut rows)
    }
    pub(crate) fn select_links_by_target(&self, target_url: &str) -> FieldResult<Vec<Link>> {
        let connection = self.pool.get()?;

        let mut select =
            connection.prepare_cached(include_str!("../sql/select_links_by_target.sql"))?;
        let mut rows = select.query_named(named_params! {":target_url": target_url})?;
        Link::decode_rows(&mut rows)
    }
    pub(crate) fn select_tags_by_target(&self, target_url: &str) -> FieldResult<Vec<Tag>> {
        let connection = self.pool.get()?;

        let mut select =
            connection.prepare_cached(include_str!("../sql/select_tags_by_target.sql"))?;
        let mut rows = select.query_named(named_params! {":target_url": target_url})?;
        Tag::decode_rows(&mut rows)
    }
    pub(crate) fn select_tags_by_name(&self, name: &str) -> FieldResult<Vec<Tag>> {
        let connection = self.pool.get()?;

        let mut select =
            connection.prepare_cached(include_str!("../sql/select_tags_by_name.sql"))?;
        let mut rows = select.query_named(named_params! {":name": name})?;
        Tag::decode_rows(&mut rows)
    }
}
