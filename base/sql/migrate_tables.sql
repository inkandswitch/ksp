ALTER TABLE resources ADD COLUMN icon Text;
ALTER TABLE resources ADD COLUMN image Text;
DROP VIEW view_links;
CREATE VIEW IF NOT EXISTS
  view_links
AS
SELECT
  referrer_url,
  resources.title as referrer_title,
  resources.description as referrer_description,
  resources.cid as referrer_cid,
  resources.icon as referrer_icon,
  resources.image as referrer_image,
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
  resources.icon as referrer_icon,
  resources.image as referrer_image,
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

PRAGMA user_version = 1;