-- This file should undo anything in `up.sql`
ALTER TABLE odoo_call_log ALTER COLUMN param5 TYPE varchar(1000);
ALTER TABLE odoo_call_log ALTER COLUMN param6 TYPE varchar(1000);