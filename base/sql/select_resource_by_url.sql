SELECT cid, title, description, icon, image
FROM resources
WHERE url = :url
LIMIT 1;
