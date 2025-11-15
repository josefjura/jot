-- Add migration script here

CREATE TABLE TEMP_NOTES (
    id INTEGER PRIMARY KEY AUTOINCREMENT NOT NULL,
    content TEXT NOT NULL,
		tags TEXT NOT NULL,
    user_id INTEGER NOT NULL,
    created_at DATETIME DEFAULT CURRENT_TIMESTAMP,
    updated_at DATETIME DEFAULT CURRENT_TIMESTAMP,
    FOREIGN KEY (user_id) REFERENCES users (id) ON DELETE CASCADE
);

INSERT INTO TEMP_NOTES (content, tags, user_id, created_at, updated_at)
SELECT content, '', user_id, created_at, updated_at
FROM notes;

DROP TABLE notes;

ALTER TABLE TEMP_NOTES RENAME TO notes;