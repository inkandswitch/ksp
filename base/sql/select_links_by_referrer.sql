SELECT
  kind,
  referrer_url,
  referrer_cid,
  referrer_title,
  referrer_description,
  referrer_icon,
  referrer_image,
  referrer_fragment,
  referrer_location,
  
  target_url,
  identifier,
  name,
  title
FROM
  view_links
WHERE
  referrer_url = :referrer_url;