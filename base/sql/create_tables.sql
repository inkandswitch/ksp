PRAGMA foreign_keys = ON;

CREATE TABLE IF NOT EXISTS resources (
  url NOT NULL,
  title Text,
  description Text,
  cid Text,

  PRIMARY KEY (url)
)
WITHOUT ROWID;

CREATE TABLE IF NOT EXISTS inline_links(
  referrer_url Text,
  referrer_fragment Text,
  referrer_location Text,
  
  target_url Text NOT NULL,
  name Text NOT NULL,
  title Text,

  FOREIGN KEY(referrer_url) REFERENCES resources(url)
);

CREATE INDEX IF NOT EXISTS inline_links_idx_target_url ON
  inline_links (target_url);
CREATE INDEX IF NOT EXISTS inline_links_idx_referrer_url ON
  inline_links (referrer_url);

CREATE TABLE IF NOT EXISTS reference_links (
  referrer_url Text,
  referrer_fragment Text,
  referrer_location Text,

  target_url Text NOT NULL,
  identifier Text NOT NULL,
  name Text NOT NULL,
  title Text,

  FOREIGN KEY(referrer_url) REFERENCES resources(url)
);
CREATE INDEX IF NOT EXISTS reference_links_idx_target_url ON
  reference_links (target_url);
CREATE INDEX IF NOT EXISTS reference_links_idx_referrer_url ON
  reference_links (referrer_url);
CREATE INDEX IF NOT EXISTS reference_links_idx_identifier ON
  reference_links (identifier);


CREATE TABLE IF NOT EXISTS tags (
  target_url Text,
  name Text NOT NULL,
  target_fragment Text,
  target_location Text,

  FOREIGN KEY (target_url) REFERENCES resources(url),
  PRIMARY KEY (target_url, name, target_fragment)
)
WITHOUT ROWID;
CREATE INDEX IF NOT EXISTS tags_idx_target_url on tags (target_url);
CREATE INDEX IF NOT EXISTS tags_idx_tags on tags (target_url);


CREATE VIEW IF NOT EXISTS
  view_links
AS
SELECT
  referrer_url,
  resources.title as referrer_title,
  resources.description as referrer_description,
  resources.cid as referrer_cid,
  referrer_fragment,
  referrer_location,

  target_url,
  NULL as identifier,
  name,
  inline_links.title as title,
  0 as kind
FROM
  inline_links
INNER JOIN
  resources
ON
  inline_links.referrer_url = resources.url

UNION

SELECT
  referrer_url,
  resources.title as referrer_title,
  resources.description as referrer_description,
  resources.cid as referrer_cid,
  referrer_fragment,
  referrer_location,
  
  target_url,
  identifier,
  name,
  reference_links.title AS title,
  1 as kind
FROM
  reference_links
INNER JOIN
  resources
ON
  reference_links.referrer_url = resources.url;


SELECT * from view_links;