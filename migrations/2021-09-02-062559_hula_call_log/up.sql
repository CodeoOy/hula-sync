CREATE TABLE hula_call_log (
  id UUID NOT NULL PRIMARY KEY,
  hula_id UUID NULL,
  odoo_id INT NOT NULL,
  url VARCHAR(1000) NOT NULL,
  verb VARCHAR(15) NOT NULL,
  payload VARCHAR(10000),
  status INT,
  response VARCHAR(10000),
  inserted_by VARCHAR(100) NOT NULL,
  inserted_at TIMESTAMP NOT NULL,
  updated_by VARCHAR(100) NOT NULL,
  updated_at TIMESTAMP NOT NULL,
  updated_count SMALLINT NOT NULL
);

CREATE INDEX hula_call_log_hula_id ON hula_call_log (hula_id);
CREATE INDEX hula_call_log_odoo_id ON hula_call_log (odoo_id);

SELECT hula_manage_table('hula_call_log');
