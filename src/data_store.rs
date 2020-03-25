use dirs;
use juniper::FieldResult;
use r2d2_sqlite::SqliteConnectionManager;
use rusqlite::{params, Connection};
use std::{include_str, io};

pub type DecodeResult<T> = Result<T, rusqlite::Error>;

trait RowDecoder
where
  Self: std::marker::Sized,
{
  fn decode_row(row: &rusqlite::Row<'_>) -> DecodeResult<Self>;
}

pub struct DataStore {
  pool: r2d2::Pool<SqliteConnectionManager>,
}

pub enum LinkKind {
  Inline = 0,
  Reference = 1,
}

pub struct Link {
  pub(crate) kind: LinkKind,
  pub(crate) referrer_url: String,
  pub(crate) target_url: String,
  pub(crate) name: String,
  pub(crate) title: String,
  pub(crate) identifier: Option<String>,
}

impl RowDecoder for Link {
  fn decode_row(row: &rusqlite::Row<'_>) -> DecodeResult<Self> {
    let (referrer_url, target_url, identifier, name, title, kind) = (
      row.get(0)?,
      row.get(1)?,
      row.get(2)?,
      row.get(3)?,
      row.get(4)?,
      row.get(5)?,
    );

    Ok(Link {
      kind: match kind {
        0 => LinkKind::Inline,
        1 => LinkKind::Reference,
        _ => LinkKind::Inline,
      },
      referrer_url,
      target_url,
      identifier,
      name,
      title,
    })
  }
}

pub struct Tag {
  pub(crate) tag: String,
  pub(crate) target_url: String,
}

impl RowDecoder for Tag {
  fn decode_row(row: &rusqlite::Row<'_>) -> DecodeResult<Self> {
    let (target_url, tag) = (row.get(0)?, row.get(1)?);

    Ok(Tag {
      target_url: target_url,
      tag: tag,
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
    let manager = SqliteConnectionManager::file(&path).with_init(DataStore::init_tables);
    let pool = r2d2::Pool::new(manager).expect("Failed to initialize connection manager");
    let store = DataStore { pool: pool };
    println!("Data base was initialized at {:?}", path.to_str());
    Ok(store)
  }
  pub(crate) fn init_tables(connection: &mut Connection) -> Result<(), rusqlite::Error> {
    connection.execute_batch(include_str!("init_tables.sql"))
  }
  pub(crate) fn links_by_referrer(&self, referrer_url: &str) -> FieldResult<Vec<Link>> {
    let connection = self.pool.get()?;
    let mut select = connection.prepare_cached(include_str!("links_by_referrer.sql"))?;
    let mut rows = select.query(params![referrer_url])?;
    let records = decode_rows(&mut rows, Link::decode_row)?;
    Ok(records)
  }
  pub(crate) fn links_by_target(&self, target_url: &str) -> FieldResult<Vec<Link>> {
    let connection = self.pool.get()?;

    let mut select = connection.prepare_cached(include_str!("links_by_target.sql"))?;
    let mut rows = select.query(params![target_url])?;
    let records = decode_rows(&mut rows, Link::decode_row)?;
    Ok(records)
  }
  pub(crate) fn tags_by_target(&self, url: &str) -> FieldResult<Vec<Tag>> {
    let connection = self.pool.get()?;

    let mut select = connection.prepare_cached(include_str!("tags_by_target.sql"))?;
    let mut rows = select.query(params![url])?;
    let records = decode_rows(&mut rows, Tag::decode_row)?;
    Ok(records)
  }
}

fn decode_rows<T, F>(
  rows: &mut rusqlite::Rows<'_>,
  decode_row: F,
) -> Result<Vec<T>, rusqlite::Error>
where
  F: Fn(&rusqlite::Row<'_>) -> Result<T, rusqlite::Error>,
{
  let mut records = Vec::new();
  while let Some(row) = rows.next()? {
    records.push(decode_row(&row)?);
  }
  Ok(records)
}
