use super::super::schema::*;
use serde::{Deserialize, Serialize};
//use crate::schema::invitations::password_plain;

#[derive(Debug, Serialize, Deserialize, Queryable, Insertable)]
#[table_name = "hubspot_projects"]
pub struct HubspotProject {
	pub id: uuid::Uuid,
	pub hula_id: uuid::Uuid,
	pub hubspot_id: String,
	pub name: String,
	pub updated_by: String,
}