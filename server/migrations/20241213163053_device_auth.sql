-- Table that can hold data for auth attempts (id, expire_date, device_code, token)
CREATE TABLE device_auth (
    id INTEGER PRIMARY KEY AUTOINCREMENT NOT NULL,
    expire_date TEXT NOT NULL DEFAULT CURRENT_TIMESTAMP,
    device_code TEXT NOT NULL,
    token TEXT NULL
);