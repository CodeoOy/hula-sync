-- Your SQL goes here
CREATE TABLE hubspot_projects (
  id UUID NOT NULL PRIMARY KEY,
  hula_id UUID NOT NULL,
  hubspot_id VARCHAR(100) NOT NULL,
  name VARCHAR(100) NOT NULL,
  inserted_by VARCHAR(100) NOT NULL,
  inserted_at TIMESTAMP NOT NULL,
  updated_by VARCHAR(100) NOT NULL,
  updated_at TIMESTAMP NOT NULL,
  updated_count SMALLINT NOT NULL
);

CREATE UNIQUE INDEX hubspot_projects_hula_id ON hubspot_projects (hula_id);
CREATE UNIQUE INDEX hubspot_projects_hubspot_id ON hubspot_projects (hubspot_id);

SELECT hula_manage_table('hubspot_projects');

