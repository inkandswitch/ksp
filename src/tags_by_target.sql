SELECT
  target_url,
  tag
FROM
  tags
WHERE
  target_url = ?1;