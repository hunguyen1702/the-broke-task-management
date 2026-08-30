ALTER TABLE tasks ADD COLUMN archive_reason TEXT
    CHECK (
        (archived = 0 AND archive_reason IS NULL) OR
        (archived = 1 AND archive_reason IS NOT NULL AND length(trim(archive_reason)) > 0)
    );
