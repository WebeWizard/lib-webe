CREATE TABLE webe_accounts (
  id BIGINT UNSIGNED NOT NULL PRIMARY KEY,
  email VARCHAR(100) NOT NULL UNIQUE,
  secret TINYTEXT NOT NULL,
  secret_timeout INT UNSIGNED NOT NULL,
  verify_code CHAR(30),
  verify_timeout INT UNSIGNED /* Seconds since UNIX EPOCH */
);


CREATE TABLE webe_sessions (
  token CHAR(30) NOT NULL PRIMARY KEY, /* TODO: 30 is arbitrary, any reason to change it? */
  account_id BIGINT UNSIGNED NOT NULL,
  timeout INT UNSIGNED NOT NULL, /* Seconds since UNIX EPOCH */
  FOREIGN KEY (account_id)
    REFERENCES webe_accounts(id)
    ON DELETE CASCADE
);