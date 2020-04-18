pub use crate::data::Mutations;
use crate::data::{
    InputResource, Link, LinkKind, Open, Query, Resource, ResourceInfo, SimilarResource, Tag,
};
use crate::store::DataStore;
pub use juniper::FieldError;
use juniper::{FieldResult, RootNode};
use log;
use open;
use std::io;

#[derive(Debug, Clone)]
pub struct State {
    pub store: DataStore,
}
impl State {
    pub fn new() -> io::Result<Self> {
        let store = DataStore::open()?;

        Ok(State { store })
    }
}
impl juniper::Context for State {}

/// Resource tag. For files resources on MacOS/iOS that roughly translates to file / directory tag.
// For web resources that roughly translates to bookmark tags.
#[juniper::graphql_object(Context = State)]
impl Tag {
    /// tag name
    fn name(&self) -> &str {
        &self.name
    }

    fn fragment(&self) -> Option<String> {
        self.target_fragment.clone()
    }

    fn location(&self) -> Option<String> {
        self.target_location.clone()
    }

    fn target(&self) -> Resource {
        Resource::from(&self.target_url)
    }
}

/// Represents an inline link in markdown file.
#[juniper::graphql_object(Context = State)]
impl Link {
    /// link kind in markdown terms which is either reference link or an
    /// iniline link.
    fn kind(&self) -> LinkKind {
        self.kind
    }
    /// Name that link was encountered by
    fn name(&self) -> &str {
        &self.name
    }
    /// Titile that link was encountered by
    fn title(&self) -> &str {
        &self.title
    }

    /// Label it is referred by
    /// e.g. In [Local-first software][local-first] it is "local-first"
    fn identifier(&self) -> Option<&String> {
        self.identifier.as_ref()
    }

    /// Target resource of the link
    async fn target(&self) -> Resource {
        Resource::from(&self.target_url)
    }
    /// Referrer resource
    async fn referrer(&self) -> Resource {
        Resource {
            url: self.referrer_url.clone(),
            info: Some(ResourceInfo {
                cid: self.referrer_cid.clone(),
                title: self.referrer_title.clone(),
                description: self.referrer_description.clone(),
                icon: self.referrer_icon.clone(),
                image: self.referrer_image.clone(),
            }),
        }
    }
    /// Fragment of the resource content where link was discovered.
    fn fragment(&self) -> Option<&String> {
        self.referrer_fragment.as_ref()
    }
    /// Location in the resource document where link was discovered.
    fn location(&self) -> Option<&String> {
        self.referrer_location.as_ref()
    }
}

#[juniper::graphql_object(Context = State)]
impl Resource {
    /// URL of the resource
    fn url(&self) -> &str {
        &self.url
    }
    /// information containing general information about the resource, kind of
    /// a web-card for this resource.
    async fn info(&self, state: &State) -> ResourceInfo {
        if let Some(info) = &self.info {
            info.clone()
        } else {
            if let Ok(info) = state.store.find_resource_by_url(&self.url).await {
                info
            } else {
                ResourceInfo {
                    title: self.url.split("/").last().unwrap_or("").to_string(),
                    description: format!(""),
                    cid: None,
                    icon: None,
                    image: None,
                }
            }
        }
    }

    /// Resources this document links to.
    async fn links(&self, state: &State) -> FieldResult<Vec<Link>> {
        state.store.find_links_by_referrer(&self.url).await
    }

    /// Resources that link to this document.
    async fn backLinks(&self, state: &State) -> FieldResult<Vec<Link>> {
        state.store.find_links_by_target(&self.url).await
    }

    /// Tag associated to this document.
    async fn tags(&self, state: &State) -> FieldResult<Vec<Tag>> {
        state.store.find_tags_by_target(&self.url).await
    }

    // Resources similar to this one.
    async fn similar(&self, _state: &State) -> Vec<SimilarResource> {
        vec![]
    }
}

#[juniper::graphql_object(Context = State)]
impl SimilarResource {
    /// Other resource it is similar to.
    async fn target(&self) -> Resource {
        Resource::from(self.target.clone())
    }
}

#[juniper::graphql_object(Context = State)]
impl Query {
    /// gives a resource for the given url.
    async fn resource(_state: &State, url: String) -> Resource {
        Resource::from(url)
    }
    /// finds tags for the given name.
    async fn tags(state: &State, name: String) -> FieldResult<Vec<Tag>> {
        state.store.find_tags_by_name(&name).await
    }
}

impl Mutations {
    /// Injests resource into knowledge base.
    pub async fn ingest(state: &State, input: InputResource) -> FieldResult<Resource> {
        log::info!("Ingesting resource {:}", input.url);
        let resource = state.store.insert_resource(&input)?;

        if let Some(tags) = input.tags {
            state.store.insert_tags(&input.url, &tags)?;
        }
        if let Some(links) = input.links {
            state.store.insert_links(&input.url, &links)?;
        }
        log::info!("Resource was ingested {:}", input.url);

        Ok(resource)
    }
}

#[juniper::graphql_object(Context = State)]
impl Mutations {
    async fn ingest(state: &State, resource: InputResource) -> FieldResult<Resource> {
        Mutations::ingest(state, resource).await
    }
    async fn open(_state: &State, url: String) -> Open {
        log::info!("Opening a resource {:}", url);
        if let Ok(status) = open::that(url) {
            Open {
                open_ok: true,
                exit_ok: status.success(),
                code: status.code(),
            }
        } else {
            Open {
                open_ok: false,
                exit_ok: false,
                code: None,
            }
        }
    }
}

#[derive(Debug)]
pub struct Schema {
    pub root: RootNode<'static, Query, Mutations>,
}
impl Schema {
    pub fn new() -> Schema {
        Schema {
            root: RootNode::new(Query, Mutations),
        }
    }
}
