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