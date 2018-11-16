CREATE TABLE users (
  id INTEGER NOT NULL PRIMARY KEY,
  login_provider TEXT,
  login TEXT NOT NULL,
  name TEXT NOT NULL,

  UNIQUE(login)
);

CREATE TABLE sessions (
  token TEXT NOT NULL PRIMARY KEY,
  expires BIGINT NOT NULL,
  user_id INTEGER NOT NULL
);

CREATE TABLE oauth_states (
  state TEXT PRIMARY KEY
);