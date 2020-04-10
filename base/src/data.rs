use juniper;
pub use std::convert::From;

#[derive(juniper::GraphQLEnum, Clone, Copy, Debug)]
pub enum LinkKind {
    Inline = 0,
    Reference = 1,
}

#[derive(Clone, Debug)]
pub struct Link {
    pub kind: LinkKind,
    pub referrer_url: String,
    pub referrer_cid: Option<String>,
    pub referrer_title: String,
    pub referrer_description: String,

    pub referrer_fragment: Option<String>,
    pub referrer_location: Option<String>,

    pub target_url: String,
    pub name: String,
    pub title: String,
    pub identifier: Option<String>,
}

#[derive(Clone, Debug)]
pub struct Tag {
    pub name: String,
    pub target_url: String,
    pub target_fragment: Option<String>,
    pub target_location: Option<String>,
}

#[derive(Debug, Clone)]
pub struct Resource {
    pub url: String,
    pub info: Option<ResourceInfo>,
}

#[derive(juniper::GraphQLObject, Debug, Clone)]
pub struct ResourceInfo {
    pub title: String,
    pub description: String,
    pub cid: Option<String>,
}

#[derive(Clone, Debug)]
pub struct SimilarResource {
    // URL of the similar resource
    pub target: String,
}

#[derive(juniper::GraphQLInputObject, Clone, Debug)]
pub struct InputLink {
    #[graphql(name = "targetURL")]
    pub target_url: String,

    pub referrer_fragment: Option<String>,
    pub referrer_location: Option<String>,

    pub kind: LinkKind,
    pub name: String,
    pub title: String,
    pub identifier: Option<String>,
}

#[derive(juniper::GraphQLInputObject, Clone, Debug)]
pub struct InputTag {
    pub name: String,
    pub target_fragment: Option<String>,
    pub target_location: Option<String>,
}

#[derive(juniper::GraphQLInputObject, Clone, Debug)]
pub struct InputResource {
    pub url: String,
    pub cid: Option<String>,
    pub title: String,
    pub description: String,
    pub links: Option<Vec<InputLink>>,
    pub tags: Option<Vec<InputTag>>,
}

impl From<String> for Resource {
    fn from(url: String) -> Self {
        Resource {
            url: url,
            info: None,
        }
    }
}
impl From<&str> for Resource {
    fn from(url: &str) -> Self {
        Resource {
            url: url.to_string(),
            info: None,
        }
    }
}

impl From<&String> for Resource {
    fn from(url: &String) -> Self {
        Resource {
            url: url.to_string(),
            info: None,
        }
    }
}

impl From<InputResource> for Resource {
    fn from(input: InputResource) -> Self {
        Resource {
            url: input.url,
            info: Some(ResourceInfo {
                cid: input.cid,
                title: input.title,
                description: input.description,
            }),
        }
    }
}

impl From<&InputResource> for Resource {
    fn from(input: &InputResource) -> Self {
        Resource {
            url: input.url.clone(),
            info: Some(ResourceInfo {
                cid: input.cid.clone(),
                title: input.title.clone(),
                description: input.description.clone(),
            }),
        }
    }
}

impl From<String> for InputTag {
    fn from(name: String) -> Self {
        InputTag {
            name: name,
            target_fragment: None,
            target_location: None,
        }
    }
}

pub struct Query;
pub struct Mutations;
