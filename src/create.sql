PRAGMA foreign_keys = ON;

CREATE TABLE IF NOT EXISTS inline_links(
  referrer_url Text NOT NULL,
  target_url Text NOT NULL,
  name Text,
  title Text
);

CREATE INDEX IF NOT EXISTS idx_inline_links ON
  inline_links (target_url, referrer_url, name, title);

CREATE TABLE IF NOT EXISTS reference_links (
  referrer_url Text NOT NULL,
  target_url Text NOT NULL,
  identifier Text NOT NULL,
  name Text,
  title Text
);
CREATE INDEX IF NOT EXISTS idx_referrer_links ON
  reference_links (target_url, referrer_url, identifier, name, title);


CREATE TABLE IF NOT EXISTS tags (
  target_url Text NOT NULL,
  tag Text NOT NULL,
  PRIMARY KEY (target_url, tag)
)
WITHOUT ROWID;
CREATE INDEX idx_tags on tags (target_url, tag);

  

INSERT INTO inline_links
  (
    referrer_url,
    target_url,
    name,
    title
  )
VALUES
  (
    'file:///Users/gozala/Documents/Notes/farm.md',
    'https://automerge.github.io/',
    'automerge',
    'A JSON-like data structure (a CRDT) for collaborative applications in JS.'
  );

INSERT INTO reference_links
  (
    referrer_url,
    target_url,
    identifier,
    name,
    title
  )
VALUES
  (
    'file:///Users/gozala/Documents/Notes/farm.md',
    'https://www.inkandswitch.com/local-first.html',
    'local-first',
    'local-first principles',
    'Local-first software: You own your data, in spite of the cloud'
  );


INSERT INTO tags
  (
    target_url,
    tag
  )
VALUES
  (
    'file:///Users/gozala/Documents/Notes/farm.md',
    'end-user programming'
  ),
  (
    'file:///Users/gozala/Documents/Notes/farm.md',
    'corkboard'
  );



--   ("https://www.inkandswitch.com/local-first.html");

-- SELECT res_id FROM resources
--   WHERE resources.url = "file:///Projects/research-trails/@gozala/02-03-2020.md"
--   LIMIT 1;




-- DELETE from resources;

-- INSERT INTO resources (url)
-- VALUES("file:///Projects/research-trails/@gozala/02-03-2020.md");

SELECT
  referrer_url,
  target_url,
  NULL as identifier,
  name,
  title,
  0 as kind
FROM
  inline_links

UNION

SELECT
  referrer_url,
  target_url,
  identifier,
  name,
  title,
  1 as kind
FROM
  reference_links
;


SELECT
  target_url,
  tag
FROM
  tags;