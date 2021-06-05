use super::super::schema::*;
use diesel::{r2d2::ConnectionManager, PgConnection};
use serde::{Deserialize, Serialize};
//use crate::schema::invitations::password_plain;

pub type Pool = r2d2::Pool<ConnectionManager<PgConnection>>;

#[derive(Debug, Serialize, Deserialize, Queryable, Insertable)]
#[table_name = "odoo_projects"]
pub struct OdooProject {
	pub id: uuid::Uuid,
	pub hula_id: uuid::Uuid,
	pub odoo_id: i32,
	pub name: String,
	pub updated_by: String,
}