use juniper;

#[derive(juniper::GraphQLEnum, Clone, Copy, Debug)]
pub enum LinkKind {
  Inline = 0,
  Reference = 1,
}

#[derive(Clone, Debug)]
pub struct Link {
  pub kind: LinkKind,
  pub referrer_url: String,
  pub target_url: String,
  pub name: String,
  pub title: String,
  pub identifier: Option<String>,
}

#[derive(Clone, Debug)]
pub struct Tag {
  pub tag: String,
  pub target_url: String,
}

#[derive(Clone, Debug)]
pub struct Resource {
  pub url: String,
}

#[derive(Clone, Debug)]
pub struct SimilarResource {
  // URL of the similar resource
  pub target: String,
}

#[derive(juniper::GraphQLInputObject, Clone, Debug)]
pub struct InputLink {
  pub target_url: String,
  pub kind: LinkKind,
  pub name: String,
  pub title: String,
  pub identifier: Option<String>,
}

#[derive(juniper::GraphQLInputObject, Clone, Debug)]
pub struct InputResource {
  pub url: String,
  pub links: Option<Vec<InputLink>>,
  pub tags: Option<Vec<String>>,
}

pub struct Query;
pub struct Mutations;
