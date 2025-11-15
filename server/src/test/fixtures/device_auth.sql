INSERT INTO device_auth (device_code) VALUES ('code-without-token');
INSERT INTO device_auth (device_code, token) VALUES ('code-with-token', 'mock-token');

-- CREATE TABLE device_auth (
--     id SERIAL PRIMARY KEY,
--     expire_date TIMESTAMP NOT NULL DEFAULT CURRENT_TIMESTAMP,
--     device_code VARCHAR(255) NOT NULL,
--     token VARCHAR(255) NULL
-- );