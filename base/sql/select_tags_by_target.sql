SELECT
  target_url,
  name,
  target_fragment,
  target_location
FROM
  tags
WHERE
  target_url = :target_url;