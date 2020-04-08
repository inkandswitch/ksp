INSERT OR IGNORE INTO resources
  (url, title, description, cid)
VALUES
  (:url, :title, :description, :cid);
