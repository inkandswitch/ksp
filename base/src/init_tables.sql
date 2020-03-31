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
CREATE INDEX IF NOT EXISTS idx_tags on tags (target_url, tag);