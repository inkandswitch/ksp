SELECT
  referrer_url,
  target_url,
  NULL as identifier,
  name,
  title,
  0 as kind
FROM
  inline_links
WHERE
  target_url = ?1

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
WHERE
  target_url = ?1;