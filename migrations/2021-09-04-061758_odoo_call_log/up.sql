CREATE TABLE odoo_call_log (
  id UUID NOT NULL PRIMARY KEY,
  script VARCHAR(1000) NOT NULL,
  param1 VARCHAR(1000) NULL,
  param2 VARCHAR(1000) NULL,
  param3 VARCHAR(1000) NULL,
  param4 VARCHAR(1000) NULL,
  param5 VARCHAR(1000) NULL,
  param6 VARCHAR(1000) NULL,
  ok BOOLEAN NULL,
  response VARCHAR(10000),
  inserted_by VARCHAR(100) NOT NULL,
  inserted_at TIMESTAMP NOT NULL,
  updated_by VARCHAR(100) NOT NULL,
  updated_at TIMESTAMP NOT NULL,
  updated_count SMALLINT NOT NULL
);

SELECT hula_manage_table('odoo_call_log');
