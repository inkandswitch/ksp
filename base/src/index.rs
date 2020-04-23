use crate::data::SimilarResource;
use log;
use std::convert::From;
use std::fmt;
use std::path::Path;
use std::string::FromUtf8Error;
use std::sync::{Arc, PoisonError, RwLock, RwLockReadGuard, RwLockWriteGuard};
use stopwords::{Stopwords, NLTK};
use tantivy::collector::TopDocs;
use tantivy::directory;
use tantivy::schema;
use tantivy::schema::{IndexRecordOption, TextFieldIndexing, TextOptions};
use tantivy::tokenizer;
use tantivy::{Index, IndexReader, IndexWriter, Opstamp};
use tique::topterms::{Keywords, TopTerms};

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
    pub fn tokenizer() -> tokenizer::TextAnalyzer {
        let words = NLTK::stopwords(stopwords::Language::English)
            .unwrap()
            .iter()
            .map(|word| word.to_string())
            .collect();

        tokenizer::TextAnalyzer::from(tokenizer::SimpleTokenizer)
            .filter(tokenizer::RemoveLongFilter::limit(40))
            .filter(tokenizer::LowerCaser)
            .filter(tokenizer::StopWordFilter::remove(words))
        // Disable stemmer as keywords appear to be stems instead of
        // actual terms.
        // .filter(tokenizer::Stemmer::new(tokenizer::Language::English))
    }
    pub fn indexer() -> TextFieldIndexing {
        TextFieldIndexing::default()
            .set_tokenizer("en_with_stopwords")
            .set_index_option(IndexRecordOption::WithFreqsAndPositions)
    }
    pub fn text_options() -> TextOptions {
        schema::TextOptions::default().set_indexing_options(Schema::indexer())
    }
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
        let title = schema.add_text_field("title", Schema::text_options());

        // Field `body` corresponds to content of the resource. It will have
        // full-text search, but not an ability to reconstruct it.
        let body = schema.add_text_field("body", Schema::text_options().set_stored());

        Schema {
            url,
            title,
            body,
            schema: schema.build(),
        }
    }

    fn index(&self, path: &Path) -> Result<Index, Error> {
        let directory = tantivy::directory::MmapDirectory::open(path)?;
        let index = tantivy::Index::open_or_create(directory, self.schema.clone())?;

        index
            .tokenizers()
            .register("en_with_stopwords", Schema::tokenizer());

        Ok(index)
    }

    fn document(&self, url: &str, title: &str, body: &str) -> Result<schema::Document, Error> {
        let mut document = schema::Document::new();
        document.add_text(self.url, url);
        document.add_text(self.title, title);
        document.add_text(self.body, body);
        Ok(document)
    }

    fn document_url(&self, doc: schema::Document) -> Result<String, Error> {
        let value = doc.get_first(self.url).ok_or(Error::URLReadError)?;
        value.text().map(String::from).ok_or(Error::URLReadError)
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
    pub fn extract_keywords(&self, input: &str, limit: usize) -> Keywords {
        self.topterms.extract(limit, input)
    }
    pub fn search_with_keywords(
        &self,
        keywords: &Keywords,
        limit: usize,
    ) -> Result<Vec<SimilarResource>, Error> {
        let searcher = self.reader.searcher();
        let top_docs =
            searcher.search(&keywords.clone().into_query(), &TopDocs::with_limit(limit))?;
        let mut similar = Vec::new();
        for (score, address) in top_docs {
            log::info!("Found match {:?} {:?}", &address, &score);
            let doc = searcher.doc(address)?;
            log::info!("Doc maps to {:?}", doc);
            let target_url = self.schema.document_url(doc)?;
            similar.push(SimilarResource {
                target_url,
                similarity_score: score,
            });
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
    OpenIndexError(directory::error::OpenDirectoryError),
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
            Error::OpenIndexError(error) => error.fmt(f),
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

impl From<directory::error::OpenDirectoryError> for Error {
    fn from(error: directory::error::OpenDirectoryError) -> Self {
        Error::OpenIndexError(error)
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
