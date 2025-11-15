INSERT INTO
    notes (id, content, tags, user_id)
VALUES (1, 'test 1', 'tag1, tag2', 1);

INSERT INTO
    notes (id, content,tags, user_id)
VALUES (2, 'test 2','tag2, tag4', 1);

INSERT INTO
    notes (id, content, tags, user_id)
VALUES (3, 'test 3','tag1, tag3', 2);

INSERT INTO
    notes (id, content, tags, user_id)
VALUES (4, 'test 4','tag3, tag4', 2);

INSERT INTO
    notes (id, content, tags, user_id, created_at)
VALUES (5, 'note','tag4, tag5', 2, CURRENT_TIMESTAMP);