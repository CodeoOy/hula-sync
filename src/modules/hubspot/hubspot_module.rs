use diesel::{prelude::*, PgConnection};
use serde::{Deserialize, Serialize};

use crate::hulautils::HulaConfig;
use crate::hulautils::{get_hula_projects, insert_hula_project, update_hula_project, HulaProject};
use crate::models::hubspot_project::HubspotProject;

use std::str;
use uuid::Uuid;

#[derive(Deserialize, Debug)]
pub struct HubspotHeader {
	deals: Vec<HubspotDeal>,
}

#[allow(non_snake_case)]
#[derive(Deserialize, Debug)]
pub struct HubspotDeal {
	dealId: u64,
	properties: HubspotProperties,
}

#[derive(Deserialize, Debug)]
pub struct HubspotProperties {
	dealname: HubspotDealName,
	dealstage: HubspotDealStage,
	palvelut: Option<HubspotPalvelut>,
}

#[derive(Deserialize, Debug)]
pub struct HubspotDealName {
	value: String,
}

#[derive(Deserialize, Debug)]
pub struct HubspotDealStage {
	value: String,
}

#[derive(Deserialize, Debug)]
pub struct HubspotPalvelut {
	value: String,
}

#[derive(Serialize, Debug)]
pub struct HubspotLimit {
	after: u64,
}

pub async fn do_process(config: &HulaConfig, conn: &PgConnection) -> Result<(), String> {
	println!("Henlo world");

	let hubspot_deals = get_hubspot_deals().await;
	println!("hubspot gotten");

	let hula_projects = get_hula_projects(&config).await;
	println!("hula gotten");

	let log = get_hubspot_log(&conn);
	println!("logs gotten: {:?}", log);

	let _ = do_process2(
		&config,
		&conn,
		hubspot_deals.unwrap().deals,
		hula_projects.unwrap(),
		log.unwrap(),
	)
	.await;
	println!("ready");

	Ok(())
}

pub async fn get_hubspot_deals() -> Result<HubspotHeader, &'static str> {
	let hubspot_key = std::env::var("HUBSPOT_API_KEY").expect("HUBSPOT_API_KEY must be set");

	let request_url = format!("https://api.hubapi.com/deals/v1/deal/paged?hapikey={}&properties=dealname&properties=dealstage&properties=palvelut&limit=250",
		hubspot_key);

	println!("Calling {}", request_url);

	let client = reqwest::Client::new();
	let response = client.get(&request_url).send().await;

	let response = match response {
		Ok(file) => file,
		Err(e) => {
			println!("{:?}", e);
			return Err("1");
		}
	};

	println!("...Response is: {:?}", &response);

	let jiison = response.json().await;

	let jiison2 = match jiison {
		Ok(file) => file,
		Err(e) => {
			println!("{:?}", e);
			return Err("2");
		}
	};

	let mut header: HubspotHeader = jiison2;

	println!("...Got {}", header.deals.len());

	header
		.deals
		.retain(|x| x.properties.dealstage.value == "1479299");

	println!("...Filtered. Remaining with {}", header.deals.len());

	println!(
		"--------------------------------------------------------------- {:?}",
		header.deals
	);

	Ok(header)
}

fn get_hubspot_log(conn: &PgConnection) -> Result<Vec<HubspotProject>, String> {
	use crate::schema::hubspot_projects::dsl::hubspot_projects;
	let items = hubspot_projects
		.load::<HubspotProject>(conn)
		.expect("failed to load from db");

	println!("\nGot all logs.\n");
	return Ok(items);
}

async fn do_process2(
	config: &HulaConfig,
	conn: &PgConnection,
	deals: Vec<HubspotDeal>,
	projects: Vec<HulaProject>,
	log: Vec<HubspotProject>,
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
			let a2 = h2
				.filter(|x| x.dealId.to_string() == log1.hubspot_id)
				.next();

			if let Some(b2) = a2 {
				println!("Some(b2) = {:?}", b2);

				let palvelut = match &b2.properties.palvelut {
					Some(x) => Some(x.value.clone()),
					None => None,
				};

				if b.name != b2.properties.dealname.value || b.description != palvelut {
					println!(
						"updating {} {}",
						b.id.clone(),
						b2.properties.dealname.value.clone()
					);

					let _ = update_hula_project(
						config,
						b.id.clone(),
						b2.properties.dealname.value.clone(),
						palvelut,
					)
					.await;
				}
			}
		}
	}

	/* iterate deals, see what needs insert */
	for deal in &deals {
		println!("deal = {:?}", deal);
		let mut h = log.iter();
		if h.any(|x| x.hubspot_id == deal.dealId.to_string()) == false {
			println!("inserting {:?}", deal.properties.dealname.value);

			let palvelut = match &deal.properties.palvelut {
				Some(x) => Some(x.value.clone()),
				None => None,
			};

			let added = insert_hula_project(&config, deal.properties.dealname.value.clone(), palvelut).await;

			let my_uuid = Uuid::parse_str(&added.expect("no way")).expect("crash here");

			let _ = insert_hubspot_log(
				&conn,
				my_uuid,
				deal.dealId.to_string(),
				deal.properties.dealname.value.clone(),
			)
			.await;
		}
	}

	Ok(())
}

async fn insert_hubspot_log(
	conn: &PgConnection,
	hula_id: uuid::Uuid,
	hubspot_id: String,
	name: String,
) -> Result<(), String> {
	use crate::schema::hubspot_projects::dsl::hubspot_projects;

	let new_project = HubspotProject {
		id: uuid::Uuid::new_v4(),
		hula_id: hula_id,
		hubspot_id: hubspot_id,
		name: name.clone(),
		updated_by: "hulasync".to_string(),
	};
	println!("Inserting data");

	let rows_inserted = diesel::insert_into(hubspot_projects)
		.values(&new_project)
		.get_result::<HubspotProject>(conn);

	println!("{:?}", rows_inserted);
	if rows_inserted.is_ok() {
		println!("\nProject added successfully.\n");
		return Ok(());
	}

	return Err("failed".to_string());
}

/*
async fn update_HulaProjects(
	header: Header,
	projects: Vec<HulaProject>,
) -> Result<(), &'static str> {

	let mut p = projects.iter();

	for deal in &header.deals {
		if p.any(|x| x.name == deal.properties.dealname.value) == false {
			println!("INSERT {}", deal.properties.dealname.value);
		}
	}

	let mut d = header.deals.iter();

	for project in &projects {
		if d.any(|x| x.properties.dealname.value == project.name) == false {
			println!("DELETE {}", project.name);
		}
	}

	Ok(())
}

*/
