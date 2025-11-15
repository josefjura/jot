-- Table that can hold data for auth attempts (id, expire_date, device_code, token)
CREATE TABLE device_auth (
    id SERIAL PRIMARY KEY,
    expire_date TIMESTAMP NOT NULL DEFAULT CURRENT_TIMESTAMP,
    device_code VARCHAR(255) NOT NULL,
    token VARCHAR(255) NULL
);