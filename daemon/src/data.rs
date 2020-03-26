#[derive(juniper::GraphQLEnum, Clone, Copy)]
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

pub struct Tag {
  pub(crate) tag: String,
  pub(crate) target_url: String,
}

pub struct Resource {
  pub(crate) url: String,
}

pub struct SimilarResource {
  // URL of the similar resource
  pub(crate) target: String,
}

#[derive(juniper::GraphQLInputObject)]
pub struct InputLink {
  pub(crate) target_url: String,
  pub(crate) kind: LinkKind,
  pub(crate) name: String,
  pub(crate) title: String,
  pub(crate) identifier: Option<String>,
}

#[derive(juniper::GraphQLInputObject)]
pub struct InputResource {
  pub(crate) url: String,
  pub(crate) links: Option<Vec<InputLink>>,
  pub(crate) tags: Option<Vec<String>>,
}

pub struct Query;
pub struct Mutations;
