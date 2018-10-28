CREATE TABLE tickets (
  id INTEGER PRIMARY KEY,
  project INTEGER NOT NULL,
  number INT NOT NULL,
  state INT NOT NULL,
  title TEXT NOT NULL,
  body TEXT NOT NULL,
  ctime INT NOT NULL,

  UNIQUE (project, number)
);

CREATE TABLE projects (
  project INTEGER PRIMARY KEY NOT NULL,
  name INT NOT NULL,
  ctime INT NOT NULL
);

CREATE TABLE workflow_states (
  id INTEGER PRIMARY KEY,
  project INT NOT NULL,
  state TEXT NOT NULL,
  name TEXT NOT NULL
);

CREATE TABLE ticket_schema_fields (
  id INTEGER PRIMARY KEY,
  project INT NOT NULL,
  name TEXT NOT NULL,
  elements_mode INT NOT NULL  /* ascii char: .,?,*,+ */
);

CREATE TABLE ticket_attributes (
  id INTEGER PRIMARY KEY,
  ticket INT NOT NULL,
  field TEXT NOT NULL,
  value TEXT NOT NULL
);

CREATE INDEX idx_ticket_attributes ON ticket_attributes(ticket);
CREATE INDEX idx_ticket_schema_fields ON ticket_schema_fields(project);
CREATE INDEX idx_workflow_states ON workflow_states(project);