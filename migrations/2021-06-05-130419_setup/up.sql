drop table testdata;

-- Sets up a trigger for the given table to automatically set a column called
-- `updated_at` whenever the row is modified (unless `updated_at` was included
-- in the modified columns)
--
-- # Example
--
-- ```sql
-- CREATE TABLE users (id SERIAL PRIMARY KEY, updated_at TIMESTAMP NOT NULL DEFAULT NOW());
--
-- SELECT diesel_manage_updated_at('users');
-- ```
CREATE OR REPLACE FUNCTION hula_manage_table(_tbl regclass) RETURNS VOID AS $$
BEGIN
    EXECUTE format('CREATE TRIGGER hula_set_inserted BEFORE INSERT ON %s
                    FOR EACH ROW EXECUTE PROCEDURE hula_set_inserted()', _tbl);
    EXECUTE format('CREATE TRIGGER hula_set_updated BEFORE UPDATE ON %s
                    FOR EACH ROW EXECUTE PROCEDURE hula_set_updated()', _tbl);
END;
$$ LANGUAGE plpgsql;

CREATE OR REPLACE FUNCTION hula_set_inserted() RETURNS trigger AS $$
BEGIN
    NEW.inserted_at := current_timestamp;
    NEW.updated_at := NEW.inserted_at;
    NEW.inserted_by := NEW.updated_by;
    NEW.updated_count := 0;
    RETURN NEW;
END;
$$ LANGUAGE plpgsql;

CREATE OR REPLACE FUNCTION hula_set_updated() RETURNS trigger AS $$
BEGIN
    IF (NEW IS DISTINCT FROM OLD) THEN
        NEW.inserted_by := OLD.inserted_by;
        NEW.inserted_at := OLD.inserted_at;
        NEW.updated_at := current_timestamp;
        NEW.updated_count := OLD.updated_count + 1;
    END IF;
    RETURN NEW;
END;
$$ LANGUAGE plpgsql;

CREATE TABLE odoo_projects (
  id UUID NOT NULL PRIMARY KEY,
  hula_id UUID NOT NULL,
  odoo_id INT NOT NULL,
  name VARCHAR(100) NOT NULL,
  inserted_by VARCHAR(100) NOT NULL,
  inserted_at TIMESTAMP NOT NULL,
  updated_by VARCHAR(100) NOT NULL,
  updated_at TIMESTAMP NOT NULL,
  updated_count SMALLINT NOT NULL
);

CREATE UNIQUE INDEX odoo_projects_hula_id ON odoo_projects (hula_id);
CREATE UNIQUE INDEX odoo_projects_odoo_id ON odoo_projects (odoo_id);

SELECT hula_manage_table('odoo_projects');

