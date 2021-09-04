use super::super::schema::*;
use serde::{Deserialize, Serialize};

#[derive(Debug, Serialize, Deserialize, Queryable, Insertable)]
#[table_name = "odoo_call_log"]
pub struct OdooCallLog {
	pub id: uuid::Uuid,
	pub script: String,
	pub param1: Option<String>,
	pub param2: Option<String>,
	pub param3: Option<String>,
	pub param4: Option<String>,
	pub param5: Option<String>,
	pub param6: Option<String>,
	pub ok: bool,
	pub response: Option<String>,
	pub updated_by: String,
	pub updated_at: chrono::NaiveDateTime,
}
