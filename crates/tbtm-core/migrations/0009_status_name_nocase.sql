CREATE UNIQUE INDEX IF NOT EXISTS statuses_name_nocase
ON statuses(name COLLATE NOCASE);
