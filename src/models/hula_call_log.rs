use super::super::schema::*;
use serde::{Deserialize, Serialize};

#[derive(Debug, Serialize, Deserialize, Queryable, Insertable)]
#[table_name = "hula_call_log"]
pub struct HulaCallLog {
	pub id: uuid::Uuid,
	pub hula_id: Option<uuid::Uuid>,
	pub odoo_id: i32,
	pub url: String,
	pub verb: String,
	pub payload: String,
	pub status: i32,
	pub response: String,
	pub updated_by: String,
	pub updated_at: chrono::NaiveDateTime,
}
