SELECT
  target_url,
  name,
  target_fragment,
  target_location
FROM
  tags
WHERE
  name = :name;