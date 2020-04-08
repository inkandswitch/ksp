SELECT
  kind,
  referrer_url,
  referrer_cid,
  referrer_title,
  referrer_description,
  referrer_fragment,
  referrer_location,
  
  target_url,
  identifier,
  name,
  title
FROM
  view_links
WHERE
  target_url = :target_url;
