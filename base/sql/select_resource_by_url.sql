SELECT cid, title, description
FROM resources
WHERE url = :url
LIMIT 1;
