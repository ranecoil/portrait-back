ALTER TABLE creators
    RENAME COLUMN pw_hash TO password;
ALTER TABLE creators
    DROP COLUMN pfp,
    ADD COLUMN picture uuid
