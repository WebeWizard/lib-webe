/* Naievely choosing UUIDs as the primary key.
I recognize that there some ways to improve performance
The easiest way is to simply use TokuDB as the engine.

MariaDB also supports storing UUID in binary(16) columns with
UUID_TO_BIN etc.
*/

CREATE TABLE webe_accounts (
  id BINARY(16) NOT NULL PRIMARY KEY,
  email VARCHAR(100) NOT NULL UNIQUE,
  secret TINYTEXT NOT NULL,
  secret_timeout INT UNSIGNED NOT NULL,
  verified BOOLEAN NOT NULL,
  verify_code CHAR(30),
  verify_timeout INT UNSIGNED /* Seconds since UNIX EPOCH */
);


CREATE TABLE webe_sessions (
  token CHAR(30) NOT NULL PRIMARY KEY, /* TODO: 30 is arbitrary, any reason to change it? */
  account_id BINARY(16) NOT NULL,
  timeout INT UNSIGNED NOT NULL, /* Seconds since UNIX EPOCH */
  FOREIGN KEY (account_id)
    REFERENCES webe_accounts(id)
    ON DELETE CASCADE
);