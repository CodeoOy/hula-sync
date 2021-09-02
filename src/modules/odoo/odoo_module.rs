use chrono::NaiveDate;
use diesel::{prelude::*, PgConnection};
use log::trace;
use serde::{Deserialize, Serialize};

use crate::hulautils::{get_hula_projects, HulaProject};
use crate::hulautils::HulaConfig;
use crate::models::odoo_project::OdooProject;

use std::process::Command;

use std::str;

#[derive(Deserialize, Serialize, Debug)]
pub struct OdooProjectHeader {
	id: i32,
	name: String,
	description: String,
	visible: bool,
	needs: Vec<OdooProjectNeed>,
}

#[derive(Deserialize, Serialize, Debug)]
pub struct OdooProjectNeed {
	label: String,
	nbr: i32,
	begin: NaiveDate,
	end: Option<NaiveDate>,
	skills: Vec<OdooProjectNeedSkill>,
}

#[derive(Deserialize, Serialize, Debug)]
pub struct OdooProjectNeedSkill {
	skill: String,
	level: Option<String>,
	min_years: Option<f64>,
	mandatory: bool,
}

#[derive(Deserialize, Serialize, Debug)]
pub struct HulaProjectStructureData {
	pub name: String,
	pub is_hidden: bool,
	pub needs: Vec<HulaProjectStructureNeedData>,
}

#[derive(Deserialize, Serialize, Debug)]
pub struct HulaProjectStructureNeedData {
	pub label: String,
	pub count_of_users: i32,
	pub begin_time: chrono::NaiveDate,
	pub end_time: Option<chrono::NaiveDate>,
	pub percentage: Option<i32>,
	pub skills: Vec<HulaProjectStructureNeedSkillData>,
}

#[derive(Deserialize, Serialize, Debug)]
pub struct HulaProjectStructureNeedSkillData {
	pub skill_label: String,
	pub skillscopelevel_label: Option<String>,
	pub min_years: Option<f64>,
	pub max_years: Option<f64>,
	pub mandatory: bool,
}

impl From<&OdooProjectHeader> for HulaProjectStructureData {
	fn from(project: &OdooProjectHeader) -> HulaProjectStructureData {
		HulaProjectStructureData {
			name: project.name.clone(),
			is_hidden: !project.visible,
			needs: project
				.needs
				.iter()
				.map(|x| HulaProjectStructureNeedData {
					label: x.label.clone(),
					count_of_users: x.nbr,
					begin_time: x.begin,
					end_time: x.end,
					percentage: Some(100),
					skills: x
						.skills
						.iter()
						.map(|y| HulaProjectStructureNeedSkillData {
							skill_label: y.skill.clone(),
							skillscopelevel_label: y.level.clone(),
							min_years: y.min_years,
							max_years: None,
							mandatory: y.mandatory,
						})
						.collect(),
				})
				.collect(),
		}
	}
}

#[derive(Deserialize, Serialize, Debug)]
pub struct ProjectMatch {
	pub id: i32,
	pub matches: i32,
	pub link: String,
}

#[derive(Deserialize, Debug)]
pub struct HulaProjectStructureResponse {
	pub id: uuid::Uuid,
	pub matches: i32,
}

pub struct OdooConfig {
	pub odoo_url: String,
	pub odoo_db: String,
	pub odoo_uid: String,
	pub odoo_pw: String,
}

fn get_config() -> OdooConfig {
	let config = OdooConfig {
		odoo_url: std::env::var("ODOO_URL").expect("ODOO_URL must be set"),
		odoo_db: std::env::var("ODOO_DB").expect("ODOO_DB must be set"),
		odoo_uid: std::env::var("ODOO_USERNAME").expect("ODOO_USERNAME must be set"),
		odoo_pw: std::env::var("ODOO_PASSWORD").expect("ODOO_PASSWORD must be set"),
	};

	config
}

pub async fn do_process(config: &HulaConfig, conn: &PgConnection) -> Result<(), String> {
	trace!("Processing Odoo interface.");

	let odoo_deals = get_odoo_deals().await?;
	trace!("Got Odoo unprocessed projects: {}", odoo_deals.len());

	let hula_projects = get_hula_projects(&config).await?;
	trace!("Got Hula project descriptions: {}", hula_projects.len());

	let log = get_odoo_log(&conn).await?;
	trace!("Got Integration project descriptions: {}", log.len());

	let matches = do_process_internal(&config, &conn, odoo_deals, hula_projects, log).await?;
	trace!("Processing resulted in matches: {}", matches.len());

	put_odoo_matches(matches).await?;

	trace!("Odoo interface done.");
	Ok(())
}

async fn get_odoo_deals() -> Result<Vec<OdooProjectHeader>, String> {
	let c = get_config();

	trace!(
		"Running: python3 src/modules/odoo/python/odoo_get.py {} {} {} {}",
		&c.odoo_url,
		&c.odoo_db,
		&c.odoo_uid,
		&c.odoo_pw
	);

	let a = Command::new("python3")
		.args(&[
			"src/modules/odoo/python/odoo_get.py",
			&c.odoo_url,
			&c.odoo_db,
			&c.odoo_uid,
			&c.odoo_pw,
		])
		.output();

	let a = match a {
		Ok(x) => x,
		Err(e) => return Err(format!("Python3 failed: {}", e)),
	};
	
	let s = match str::from_utf8(&a.stdout) {
		Ok(v) => v,
		Err(e) => return Err(format!("Invalid UTF-8 sequence on stdout: {}", e)),
	};

	trace!("Output:\n{}", &s);

	let er = match str::from_utf8(&a.stderr) {
		Ok(v) => v,
		Err(e) => return Err(format!("Invalid UTF-8 sequence on stderr: {}", e)),
	};

	if er.is_empty() == false {
		trace!("Errors:\n{}", &er);
	}

	let json = //: Vec<OdooProjectHeader> =
		serde_json::from_str(s); //.expect("JSON was not well-formatted");

	let json = match json {
		Ok(v) => v,
		Err(e) => return Err(format!("JSON was not well-formatted: {}", e)),
	};
	
	Ok(json)
}

async fn put_odoo_matches(matches: Vec<ProjectMatch>) -> Result<(), String> {
	let c = get_config();

	let odoo_matches = serde_json::to_string(&matches);

	let odoo_matches = match odoo_matches {
		Ok(v) => format!("{}", v),
		Err(e) => return Err(format!("Serde failed: {}", e)),
	};

	trace!(
		"Running: python3 src/modules/odoo/python/odoo_put.py {} {} {} {} {}",
		&c.odoo_url,
		&c.odoo_db,
		&c.odoo_uid,
		&c.odoo_pw,
		&odoo_matches
	);

	let cmd = Command::new("python3")
		.args(&[
			"src/modules/odoo/python/odoo_put.py",
			&c.odoo_url,
			&c.odoo_db,
			&c.odoo_uid,
			&c.odoo_pw,
			&odoo_matches,
		])
		.output();

	let _ = match cmd {
		Ok(_) => Ok(()),
		Err(e) => Err(format!("Python3 failed: {}", e)),
	};

	Ok(())
}

async fn get_odoo_log(conn: &PgConnection) -> Result<Vec<OdooProject>, String> {
	use crate::schema::odoo_projects::dsl::odoo_projects;
	let items = odoo_projects
		.load::<OdooProject>(conn);

	let items = match items {
		Ok(items) => items,
		Err(e) => return Err(format!("get_odoo_log failed: {}", e)),
	};

	return Ok(items);
}

async fn do_process_internal(
	config: &HulaConfig,
	conn: &PgConnection,
	deals: Vec<OdooProjectHeader>,
	projects: Vec<HulaProject>,
	log: Vec<OdooProject>,
) -> Result<Vec<ProjectMatch>, String> {
	let mut matches: Vec<ProjectMatch> = vec![];

	//let c = get_config();

	/* iterate log, see what needs update */
	for log1 in &log {
		let h = projects.iter();
		let a = h.filter(|x| x.id == log1.hula_id.to_string()).next();

		if let Some(b) = a {
			let h2 = deals.iter();
			let a2 = h2.filter(|x| x.id == log1.odoo_id).next();

			if let Some(b2) = a2 {
				let updated = update_hula_project_odoo(config, b.id.clone(), b2).await;
				let updated = match updated {
					Ok(item) => item,
					Err(e) => return Err(format!("update_hula_project_odoo failed: {}", e)),
				};
				matches.push(ProjectMatch {
					id: log1.odoo_id,
					matches: updated.matches,
					link: format!("{}/app/project/{}", &config.hula_url, log1.hula_id),
				});
			}
		}
	}

	/* iterate deals, see what needs insert */
	for deal in &deals {
		let mut h = log.iter();
		if h.any(|x| x.odoo_id == deal.id) == false {
			let added = insert_hula_project_odoo(config, deal).await;
			let added = match added {
				Ok(item) => item,
				Err(e) => return Err(format!("insert_hula_project_odoo failed: {}", e)),
			};

			let my_uuid = added.id;

			let inserted = insert_odoo_log(&conn, my_uuid, deal.id, deal.name.clone()).await;
			let _ = match inserted {
				Ok(item) => item,
				Err(e) => return Err(format!("insert_odoo_log failed: {}", e)),
			};

			matches.push(ProjectMatch {
				id: deal.id,
				matches: added.matches,
				link: format!("{}/app/project/{}", &config.hula_url, &my_uuid),
			});
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

	let rows_inserted = diesel::insert_into(odoo_projects)
		.values(&new_project)
		.get_result::<OdooProject>(conn);

	if rows_inserted.is_ok() {
		return Ok(());
	}

	return Err("failed".to_string());
}

pub async fn insert_hula_project_odoo(
	config: &HulaConfig,
	header: &OdooProjectHeader,
) -> Result<HulaProjectStructureResponse, &'static str> {
	//let c = get_config();

	let request_url = format!("{}/api/projectstructures", config.hula_url);

	let client = reqwest::Client::new();

	let data: HulaProjectStructureData = header.into();

	let response = client
		.post(&request_url)
		.header("Cookie", format!("auth={}", config.cookie))
		.json(&data)
		.send()
		.await;

	let response = match response {
		Ok(file) => file,
		Err(_) => {
			return Err("1");
		}
	};

	let jiison = response.json().await;

	let jiison2 = match jiison {
		Ok(file) => file,
		Err(_) => {
			return Err("2");
		}
	};

	let hula_project: HulaProjectStructureResponse = jiison2;

	Ok(hula_project)
}

pub async fn update_hula_project_odoo(
	config: &HulaConfig,
	project_id: String,
	project: &OdooProjectHeader,
) -> Result<HulaProjectStructureResponse, &'static str> {
	//let c = get_config();

	let request_url = format!(
		"{}/api/projectstructures/{}",
		config.hula_url,
		project_id.clone()
	);

	let client = reqwest::Client::new();

	let data: HulaProjectStructureData = project.into();

	let response = client
		.put(&request_url)
		.header("Cookie", format!("auth={}", config.cookie))
		.json(&data)
		.send()
		.await;

	let response = match response {
		Ok(file) => file,
		Err(_) => {
			return Err("1");
		}
	};

	let jiison = response.json().await;

	let jiison2 = match jiison {
		Ok(file) => file,
		Err(_) => {
			return Err("2");
		}
	};

	let hula_project: HulaProjectStructureResponse = jiison2;

	Ok(hula_project)
}
