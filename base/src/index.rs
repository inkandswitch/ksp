use std::convert::From;
use std::fmt;
use std::path::Path;
use std::string::FromUtf8Error;
use std::sync::{Arc, PoisonError, RwLock, RwLockReadGuard, RwLockWriteGuard};
use tantivy::collector::TopDocs;
use tantivy::schema;
use tantivy::{Index, IndexReader, IndexWriter, Opstamp};
use tique::topterms::TopTerms;

#[derive(Clone)]
struct Schema {
    url: schema::Field,
    title: schema::Field,
    body: schema::Field,
    schema: schema::Schema,
}
impl fmt::Debug for Schema {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_struct("Schema").finish()
    }
}

impl Schema {
    pub fn new() -> Self {
        let mut schema = schema::SchemaBuilder::default();
        // Field corresponds to the resource URL. Because we want to be able
        // to see the `url` in returned documents we mark it stored.
        let url = schema.add_text_field("url", schema::STRING | schema::STORED);

        // Field corresponds to resource.info.title. Field has a `Text` flag which
        // implies that it will be tokenized and indexed, along with its term
        // frequency and term positions.
        // Field also has `STORED` flag which means that the field will also be
        // saved in a compressed row-oriented key-value store.
        let title = schema.add_text_field("title", schema::TEXT | schema::STORED);

        // Field `body` corresponds to content of the resource. It will have
        // full-text search, but not an ability to reconstruct it.
        let body = schema.add_text_field("body", schema::TEXT);

        Schema {
            url,
            title,
            body,
            schema: schema.build(),
        }
    }

    fn index(&self, path: &Path) -> Result<Index, Error> {
        let index =
            Index::create_in_dir(path, self.schema.clone()).or_else(|error| match error {
                tantivy::TantivyError::IndexAlreadyExists => Ok(Index::open_in_dir(path)?),
                _ => Err(error),
            })?;
        Ok(index)
    }

    // fn field(&self, name: &str) -> Result<schema::Field, Error> {
    //     self.schema
    //         .get_field(name)
    //         .ok_or_else(|| Error::MissingField(name.to_string()))
    // }

    fn document(&self, url: &str, title: &str, body: &str) -> Result<schema::Document, Error> {
        let mut document = schema::Document::new();
        document.add_text(self.url, url);
        document.add_text(self.title, title);
        document.add_text(self.body, body);
        Ok(document)
    }

    fn document_url(&self, doc: schema::Document) -> Result<String, Error> {
        let value = doc.get_first(self.url).ok_or(Error::URLReadError)?;

        match value {
            schema::Value::Bytes(bytes) => Ok(String::from_utf8(bytes.clone())?),
            _ => Err(Error::URLReadError),
        }
    }
}

pub struct IndexService {
    schema: Schema,
    reader: IndexReader,
    writer: Arc<RwLock<IndexWriter>>,
    topterms: TopTerms,
}

impl IndexService {
    pub fn open() -> Result<Self, Error> {
        let mut path = dirs::home_dir().expect("Unable to locate user home directory");
        path.push(".knowledge-service");
        path.push("tantivy");
        std::fs::create_dir_all(&path)?;
        IndexService::activate(&path)
    }
    pub fn activate(path: &Path) -> Result<Self, Error> {
        let schema = Schema::new();
        let index = schema.index(path)?;
        let topterms = TopTerms::new(&index, vec![schema.body])?;
        let reader = index.reader()?;
        let writer = Arc::new(RwLock::new(index.writer(50_000_000)?));
        Ok(IndexService {
            schema,
            reader,
            writer,
            topterms,
        })
    }
    pub async fn ingest(&self, url: &str, title: &str, body: &str) -> Result<Opstamp, Error> {
        let writer = self.writer.read()?;
        let doc = self.schema.document(url, title, body)?;
        Ok(writer.add_document(doc))
    }
    pub async fn commit(&self) -> Result<Opstamp, Error> {
        let mut writer = self.writer.write()?;
        Ok(writer.commit()?)
    }
    pub async fn search_similar(&self, input: &str, limit: usize) -> Result<Vec<String>, Error> {
        let keywords = self.topterms.extract(limit, input);
        let searcher = self.reader.searcher();
        let top_docs = searcher.search(&keywords.into_query(), &TopDocs::with_limit(10))?;
        let mut similar = Vec::new();
        for (_score, address) in top_docs {
            let doc = searcher.doc(address)?;
            let url = self.schema.document_url(doc)?;

            similar.push(url);
        }
        Ok(similar)
    }
}

impl fmt::Debug for IndexService {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_struct("Schema")
            .field("schema", &self.schema)
            .finish()
    }
}

#[derive(Debug)]
pub enum Error {
    IndexError(tantivy::TantivyError),
    ReadLockError,
    WriteLockError,
    URLDecodeError(FromUtf8Error),
    URLReadError,
    MissingField(String),
    IOError(std::io::Error),
}

impl std::error::Error for Error {}

impl fmt::Display for Error {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Error::IndexError(error) => error.fmt(f),
            Error::ReadLockError => write!(f, "Fiiled to create read lock on index"),
            Error::WriteLockError => write!(f, "Failed to create write lock on index"),
            Error::URLDecodeError(error) => error.fmt(f),
            Error::URLReadError => write!(f, "Was unable to read url for the indexed document"),
            Error::MissingField(name) => write!(
                f,
                "Was unable to read {} field of the indexed document",
                name
            ),
            Error::IOError(error) => error.fmt(f),
        }
    }
}

impl<'a> From<PoisonError<RwLockReadGuard<'a, IndexWriter>>> for Error {
    fn from(_: PoisonError<RwLockReadGuard<'a, IndexWriter>>) -> Error {
        Error::ReadLockError
    }
}

impl<'a> From<PoisonError<RwLockWriteGuard<'a, IndexWriter>>> for Error {
    fn from(_: PoisonError<RwLockWriteGuard<'a, IndexWriter>>) -> Self {
        Error::WriteLockError
    }
}

impl From<tantivy::TantivyError> for Error {
    fn from(error: tantivy::TantivyError) -> Self {
        Error::IndexError(error)
    }
}

impl From<FromUtf8Error> for Error {
    fn from(error: FromUtf8Error) -> Self {
        Error::URLDecodeError(error)
    }
}

impl From<std::io::Error> for Error {
    fn from(error: std::io::Error) -> Self {
        Error::IOError(error)
    }
}
