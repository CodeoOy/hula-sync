use chrono::NaiveDate;
use diesel::{prelude::*, PgConnection};
use serde::{Deserialize, Serialize};

use crate::models::odoo_project::{OdooProject};
use crate::hulautils::{HulaProject, get_hula_projects};

use std::process::Command;

use std::str;
use uuid::Uuid;

#[derive(Deserialize, Serialize, Debug)]
pub struct OdooProjectHeader {
    id: i32,
    name: String,
    description: String,
	needs: Vec<OdooProjectNeed>
}

#[derive(Deserialize, Serialize, Debug)]
pub struct OdooProjectNeed {
    label: String,
    nbr: i32,
    begin: NaiveDate,
    end: Option<NaiveDate>,
	skills: Vec<OdooProjectNeedSkill>
}

#[derive(Deserialize, Serialize, Debug)]
pub struct OdooProjectNeedSkill {
    skill: String,
    level: Option<String>,
    min_years: Option<f64>,
    mandatory: bool,
}

#[derive(Deserialize, Serialize, Debug)]
pub struct ProjectStructureData {
	pub name: String,
	pub is_hidden: bool,
	pub needs: Vec<ProjectStructureNeedData>,
}

#[derive(Deserialize, Serialize, Debug)]
pub struct ProjectStructureNeedData {
	pub label: String,
	pub count_of_users: i32,
	pub begin_time: chrono::NaiveDate,
	pub end_time: Option<chrono::NaiveDate>,
	pub percentage: Option<i32>,
	pub skills: Vec<ProjectStructureNeedSkillData>,
}

#[derive(Deserialize, Serialize, Debug)]
pub struct ProjectStructureNeedSkillData {
	pub skill_label: String,
	pub skillscopelevel_label: Option<String>,
	pub min_years: Option<f64>,
	pub max_years: Option<f64>,
	pub mandatory: bool,
}

impl From<&OdooProjectHeader> for ProjectStructureData {
	fn from(project: &OdooProjectHeader) -> ProjectStructureData {
		ProjectStructureData {
			name: project.name.clone(),
			is_hidden: false,
			needs: project.needs.iter().map(|x| ProjectStructureNeedData {
				label: x.label.clone(),
				count_of_users: x.nbr,
				begin_time: x.begin,
				end_time: x.end,
				percentage: Some(100),
				skills: x.skills.iter().map(|y| ProjectStructureNeedSkillData {
					skill_label: y.skill.clone(),
					skillscopelevel_label: y.level.clone(),
					min_years: y.min_years,
					max_years: None,
					mandatory: y.mandatory,
				}).collect()
			}).collect()
		}
	}
}

#[derive(Deserialize, Serialize, Debug)]
pub struct ProjectMatch {
	pub id: i32,
	pub matches: i32,
	pub link: String,
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

	let matches = do_process2(&conn, odoo_deals.unwrap(), hula_projects.unwrap(), log.unwrap()).await;
	println!("do_process2 done");

	let matches = match matches {
		Ok(v) => v,
		Err(e) => return Err(format!("ProjectMatches failed: {}", e)),
	};

	let m = put_odoo_matches(matches).await;
	println!("put_odoo_matches done {:?}", m);

	Ok(())
}

async fn get_odoo_deals(
) -> Result<Vec<OdooProjectHeader>, String> {

	let odoo_url =
		std::env::var("ODOO_URL").expect("ODOO_URL must be set");

	let odoo_db =
		std::env::var("ODOO_DB").expect("ODOO_DB must be set");

	let odoo_id =
		std::env::var("ODOO_USERNAME").expect("ODOO_USERNAME must be set");

	let odoo_pw =
		std::env::var("ODOO_PASSWORD").expect("ODOO_PASSWORD must be set");

	println!("python3 src/modules/odoo/python/odoo_get.py {} {} {} {}",&odoo_url, &odoo_db, &odoo_id, &odoo_pw);

	let a = Command::new("python3")
        .args(&["src/modules/odoo/python/odoo_get.py", &odoo_url, &odoo_db, &odoo_id, &odoo_pw])
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

    let json: Vec<OdooProjectHeader> =
        serde_json::from_str(s).expect("JSON was not well-formatted");

	println!("...Got {} projects.", json.len());

	Ok(json)
}

async fn put_odoo_matches(
	matches: Vec<ProjectMatch>
) -> Result<(), String> {

	let odoo_url =
		std::env::var("ODOO_URL").expect("ODOO_URL must be set");

	let odoo_db =
		std::env::var("ODOO_DB").expect("ODOO_DB must be set");

	let odoo_id =
		std::env::var("ODOO_USERNAME").expect("ODOO_USERNAME must be set");

	let odoo_pw =
		std::env::var("ODOO_PASSWORD").expect("ODOO_PASSWORD must be set");

	let odoo_matches = serde_json::to_string(&matches);

	let odoo_matches = match odoo_matches {
		Ok(v) => format!("{}", v),
		Err(e) => return Err(format!("Serde failed: {}", e)),
	};

	println!("python3 src/modules/odoo/python/odoo_put.py {} {} {} {} {}",&odoo_url, &odoo_db, &odoo_id, &odoo_pw, &odoo_matches);

	let _ = Command::new("python3")
        .args(&["src/modules/odoo/python/odoo_put.py", &odoo_url, &odoo_db, &odoo_id, &odoo_pw, &odoo_matches])
		.output()
        .expect("python3 failed to start");

	Ok(())
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
	deals: Vec<OdooProjectHeader>,
	projects: Vec<HulaProject>,
	log: Vec<OdooProject>,
) -> Result<Vec<ProjectMatch>, String> {
	println!("Henlo world");

	let mut matches :Vec<ProjectMatch> = vec!();

	let hula_url =
		std::env::var("HULA_URL").expect("HULA_URL must be set");

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
				println!("updating {} {}", b.id.clone(), b2.name.clone());
				let _ = update_hula_project_odoo(b.id.clone(), b2).await;

				matches.push(ProjectMatch {id: log1.odoo_id, matches: 3, link: format!("{}/app/project/{}", &hula_url, log1.hula_id) });
			}			
		}
	}

	/* iterate deals, see what needs insert */
	for deal in &deals {
		println!("deal = {:?}", deal);
		let mut h = log.iter(); 
		if h.any(|x| x.odoo_id == deal.id) == false {

			println!("inserting {:?}", deal.id);

			let added = insert_hula_project_odoo(deal).await;

			let my_uuid =
				Uuid::parse_str(&added.expect("no way")).expect("crash here");

			let _ = insert_odoo_log(&conn, my_uuid, deal.id, deal.name.clone()).await;
			matches.push(ProjectMatch {id: deal.id, matches: 3, link: format!("{}/app/projects/{}", &hula_url, &my_uuid) });
		}
	}

	Ok(matches)
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

pub async fn insert_hula_project_odoo(
	header: &OdooProjectHeader,
) -> Result<String, &'static str> {

	let hula_url =
		std::env::var("HULA_URL").expect("HULA_URL must be set");

	let hula_key =
		std::env::var("HULA_API_KEY").expect("HULA_API_KEY must be set");

	let request_url = format!("{}/api/projectstructures", hula_url);
    println!("Calling {}", request_url);

	let client = reqwest::Client::new();

	let data :ProjectStructureData = header.into();

	let response = client
		.post(&request_url)
		.header("Cookie", format!("auth={}", hula_key))
		.json(&data)
		.send()
		.await;
	
	let response = match response {
		Ok(file) => file,
		Err(e) => {
			println!("{:?}", e);
			return Err("1");
		},
	};
	
	println!("...Response is: {:?}", &response);
	
	let jiison = response.json().await;

	let jiison2 = match jiison {
		Ok(file) => file,
		Err(e) => {
			println!("{:?}", e);
			return Err("2");
		},
	};
	
	let hula_project: String = jiison2;
	//let project_id = hula_project.id;
	
	Ok(hula_project)
}

pub async fn update_hula_project_odoo(
	project_id: String,
	project: &OdooProjectHeader,
) -> Result<(), &'static str> {

	let hula_url =
		std::env::var("HULA_URL").expect("HULA_URL must be set");

	let hula_key =
		std::env::var("HULA_API_KEY").expect("HULA_API_KEY must be set");

	let request_url = format!("{}/api/projectstructures/{}", hula_url, project_id.clone());
    println!("Calling {}", request_url);

	let client = reqwest::Client::new();

	let data :ProjectStructureData = project.into();

	let response = client
		.put(&request_url)
		.header("Cookie", format!("auth={}", hula_key))
		.json(&data)
		.send()
		.await;
	
	let response = match response {
        Ok(file) => file,
        Err(e) => {
			println!("{:?}", e);
			return Err("1");
		},
    };

	println!("...Response is: {:?}", &response);

	Ok(())
}
