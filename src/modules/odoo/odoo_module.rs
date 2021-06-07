use diesel::{prelude::*, PgConnection};
use serde::{Deserialize};

/*use crate::errors::ServiceError;*/
use crate::models::odoo_project::{OdooProject};
use crate::hulautils::{get_hula_projects, update_hula_project, insert_hula_project, HulaProject};

use std::process::Command;

use std::str;
use uuid::Uuid;

#[derive(Deserialize, Debug)]
pub struct OdooDeal {
    id: i32,
    name: String,
}

pub async fn do_process(
	conn: &PgConnection,
) -> Result<(), String> {
	println!("Henlo world");

	let odoo_deals = get_odoo_deals().await;
	println!("odoo gotten");

	let hula_projects = get_hula_projects().await;
	println!("hula gotten");

	let log = get_odoo_log(&conn);
	println!("logs gotten: {:?}", log);

	let _ = do_process2(&conn, odoo_deals.unwrap(), hula_projects.unwrap(), log.unwrap()).await;
	println!("ready");

	Ok(())
}

async fn get_odoo_deals(
) -> Result<Vec<OdooDeal>, String> {

	let odoo_url =
		std::env::var("ODOO_URL").expect("ODOO_URL must be set");

	let odoo_db =
		std::env::var("ODOO_DB").expect("ODOO_DB must be set");

	let odoo_id =
		std::env::var("ODOO_USERNAME").expect("ODOO_USERNAME must be set");

	let odoo_pw =
		std::env::var("ODOO_PASSWORD").expect("ODOO_PASSWORD must be set");

	println!("python3 src/modules/odoo/python/odoo.py {} {} {} {}",&odoo_url, &odoo_db, &odoo_id, &odoo_pw);

	let a = Command::new("python3")
        .args(&["src/modules/odoo/python/odoo.py", &odoo_url, &odoo_db, &odoo_id, &odoo_pw])
		.output()
        .expect("python3 failed to start");
	
	let s = match str::from_utf8(&a.stdout) {
		Ok(v) => v,
		Err(e) => return Err(format!("Invalid UTF-8 sequence on stdout: {}", e)),
	};

	println!("Henlo world2");
	println!("{}", s);

	let er = match str::from_utf8(&a.stderr) {
		Ok(v) => v,
		Err(e) => return Err(format!("Invalid UTF-8 sequence on stderr: {}", e)),
	};

	println!("Errors: {}", er);

    let json: Vec<OdooDeal> =
        serde_json::from_str(s).expect("JSON was not well-formatted");

	println!("...Got {} projects.", json.len());

	Ok(json)
}

fn get_odoo_log(
	conn: &PgConnection,
) -> Result<Vec<OdooProject>, String> {

	use crate::schema::odoo_projects::dsl::odoo_projects;
	let items = odoo_projects.load::<OdooProject>(conn).expect("failed to load from db");

	println!("\nGot all logs.\n");
	return Ok(items);
}

async fn do_process2(
	conn: &PgConnection,
	deals: Vec<OdooDeal>,
	projects: Vec<HulaProject>,
	log: Vec<OdooProject>,
) -> Result<(), String> {
	println!("Henlo world");

	/* iterate log, see what needs update */
	for log1 in &log {
		println!("log1 = {:?}", log1);

		let h = projects.iter(); 
		let a = h.filter(|x| x.id == log1.hula_id.to_string()).next();

		if let Some(b) = a {
			println!("Some(b) = {:?}", b);
			let h2 = deals.iter(); 
			let a2 = h2.filter(|x| x.id == log1.odoo_id).next();

			if let Some(b2) = a2 {
				println!("Some(b2) = {:?}", b2);
				if b.name != b2.name {
					println!("updating {} {}", b.id.clone(), log1.name.clone());
					let _ = update_hula_project(b.id.clone(), log1.name.clone()).await;
				}
			}			
		}
	}

	/* iterate deals, see what needs insert */
	for deal in &deals {
		println!("deal = {:?}", deal);
		let mut h = log.iter(); 
		if h.any(|x| x.odoo_id == deal.id) == false {

			println!("inserting {:?}", deal.id);

			let added = insert_hula_project(deal.name.clone()).await;

			let my_uuid =
				Uuid::parse_str(&added.expect("no way")).expect("crash here");

			let _ = insert_odoo_log(&conn, my_uuid, deal.id, deal.name.clone()).await;
		}
	}

	Ok(())
}

async fn insert_odoo_log(
	conn: &PgConnection,
	hula_id: uuid::Uuid,
	odoo_id: i32,
	name: String,

) -> Result<(), String> {

	use crate::schema::odoo_projects::dsl::odoo_projects;

	let new_project = OdooProject {
		id: uuid::Uuid::new_v4(),
		hula_id: hula_id,
		odoo_id: odoo_id,
		name: name.clone(),
		updated_by: "hulasync".to_string(),
	};
	println!("Inserting data");

	let rows_inserted = diesel::insert_into(odoo_projects)
		.values(&new_project)
		.get_result::<OdooProject>(conn);
	
	println!("{:?}", rows_inserted);
	if rows_inserted.is_ok() {
		println!("\nProject added successfully.\n");
		return Ok(());
	}

	return Err("failed".to_string());
}
