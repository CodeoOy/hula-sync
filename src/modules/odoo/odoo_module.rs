use chrono::NaiveDate;
use diesel::{prelude::*, PgConnection};
use log::{error, trace};
use reqwest::StatusCode;
use serde::{Deserialize, Serialize};

use crate::hulautils::HulaConfig;
use crate::hulautils::{get_hula_projects, HulaProject};
use crate::models::hula_call_log::HulaCallLog;
use crate::models::odoo_call_log::OdooCallLog;
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

#[derive(Deserialize, Serialize, Debug)]
pub struct Skill {
	pub id: uuid::Uuid,
	pub label: String,
	pub aliases: Vec<String>
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

	let last_run = startup(&conn).await?;

	if let Some(last_run) = last_run {
		trace!("Last run was: {}", &last_run);
	}

	sync_skills_to_odoo(config, conn).await?;

	let odoo_deals = get_odoo_deals(&conn, last_run).await?;
	trace!("No projects from odoo.");

	if odoo_deals.len() > 0 {
		trace!("Got Odoo unprocessed projects: {}", odoo_deals.len());

		let hula_projects = get_hula_projects(&config).await?;
		trace!("Got Hula project descriptions: {}", hula_projects.len());

		let log = get_odoo_log(&conn).await?;
		trace!("Got Integration project descriptions: {}", log.len());

		let matches = do_process_internal(&config, &conn, odoo_deals, hula_projects, log).await?;
		trace!("Processing resulted in matches: {}", matches.len());

		put_odoo_matches(&conn, matches).await?;
	}

	trace!("Odoo interface done.");
	Ok(())
}

async fn sync_skills_to_odoo(config: &HulaConfig, conn: &PgConnection) -> Result<(), String> {
	let hula_skills = get_skills_from_hula(config).await?;
	put_skills_to_odoo(&hula_skills, conn).await?;
	generate_skills_to_odoo_projects(&hula_skills, conn).await?;
	Ok(())
}

async fn get_skills_from_hula(config: &HulaConfig) -> Result<Vec<Skill>, String> {
	let request_url = format!("{}/api/skills", config.hula_url);
	let client = reqwest::Client::new();
	let response = match client
		.get(&request_url)
		.header("Cookie", format!("auth={}", config.cookie))
		.send()
		.await
	{
		Ok(res) => res,
		Err(err) => return Err(err.to_string()),
	};

	if response.status() == StatusCode::NO_CONTENT {
		return Ok(Vec::new());
	}

	let data = match response.error_for_status() {
		Ok(r) => r.json::<Vec<Skill>>().await,
		Err(err) => return Err(err.to_string()),
	};

	match data {
		Ok(data) => Ok(data),
		Err(err) => Err(err.to_string()),
	}
}

async fn put_skills_to_odoo(skills: &Vec<Skill>, conn: &PgConnection) -> Result<(), String> {
	let skills_json = match serde_json::to_string(skills) {
		Ok(it) => it,
		Err(err) => return Err(err.to_string()),
	};

	let result = run_odoo_script(
		String::from("src/modules/odoo/python/odoo_put_skills.py"),
		conn,
		&[skills_json],
	)
	.await;

	match result {
		Ok(output) => println!("Following skills were created in Odoo: {}", output),
		Err(e) => return Err(e),
	};

	Ok(())
}

async fn generate_skills_to_odoo_projects(skills: &Vec<Skill>, conn: &PgConnection) -> Result<(), String> {
	let skills_json = match serde_json::to_string(skills) {
		Ok(it) => it,
		Err(err) => return Err(err.to_string()),
	};

	let result = run_odoo_script(
		String::from("src/modules/odoo/python/odoo_fill_project_skills.py"),
		conn,
		&[skills_json],
	)
	.await;

	match result {
		Ok(output) => println!(
			"Skills were generated for the following Odoo leads: {}",
			output
		),
		Err(e) => return Err(e),
	};

	Ok(())
}

pub async fn run_odoo_script(
	script_path: String,
	conn: &PgConnection,
	additional_params: &[String],
) -> Result<String, String> {
	let c = get_config();

	let mut args = Vec::from([&c.odoo_url, &c.odoo_db, &c.odoo_uid, &c.odoo_pw]);
	for param in additional_params {
		args.push(&param)
	}

	let mut cmd = script_path.clone();
	for arg in &args {
		cmd.push_str(&format!(" {}", arg))
	}

	trace!("Running: python3 {}", cmd);

	let mut python_args = Vec::from([&script_path]);
	python_args.extend(args);
	let output = Command::new("python3").args(&python_args).output();

	// stdout, stderr
	let result = match output {
		Ok(x) => match str::from_utf8(&x.stdout) {
			Ok(it) => {
				// Check and handle stderr
				let stderr: Option<String> = match str::from_utf8(&x.stderr) {
					Ok(v) => {
						if v.is_empty() {
							None
						} else {
							Some(v.to_string())
						}
					}
					Err(er) => Some(format!(
						"Invalid UTF-8 sequence on stderr: {}",
						er.to_string()
					)),
				};
				// stdout, stderr
				(Some(it.to_string()), stderr)
			}
			Err(e) => (
				None,
				Some(format!(
					"Invalid UTF-8 sequence on stdout: {}",
					e.to_string()
				)),
			),
		},
		Err(e) => (None, Some(format!("Python3 failed: {}", e.to_string()))),
	};

	let output_str = result.0;
	let error = result.1;

	if error.is_some() {
		error!("{}", error.clone().unwrap())
	}

	let ok = output_str.is_some();

	let result = match output_str.clone() {
		Some(str) => str,
		_ => {
			if error.is_some() {
				error.unwrap()
			} else {
				format!("")
			}
		}
	};

	let _ = write_odoo_call_log(
		&conn,
		&script_path,
		Some(&c.odoo_url),
		Some(&c.odoo_db),
		Some(&c.odoo_uid),
		Some(&c.odoo_pw),
		match additional_params.get(0) {
			Some(s) => Some(&s[..]),
			None => None,
		},
		match additional_params.get(1) {
			Some(s) => Some(&s[..]),
			None => None,
		},
		ok,
		Some(&result),
	)
	.await;

	Ok(output_str.unwrap())
}

async fn get_odoo_deals(
	conn: &PgConnection,
	last_run: Option<i64>,
) -> Result<Vec<OdooProjectHeader>, String> {
	let c = get_config();

	let last_run: String = match last_run {
		Some(x) => x.to_string(),
		None => "".to_string(),
	};

	trace!(
		"Running: python3 src/modules/odoo/python/odoo_get.py {} {} {} {} {}",
		&c.odoo_url,
		&c.odoo_db,
		&c.odoo_uid,
		&c.odoo_pw,
		&last_run
	);

	let a = Command::new("python3")
		.args(&[
			"src/modules/odoo/python/odoo_get.py",
			&c.odoo_url,
			&c.odoo_db,
			&c.odoo_uid,
			&c.odoo_pw,
			&last_run,
		])
		.output();

	let a = match a {
		Ok(x) => x,
		Err(e) => {
			let text = format!("Python3 failed: {}", e);

			let _ = write_odoo_call_log(
				&conn,
				"src/modules/odoo/python/odoo_get.py",
				Some(&c.odoo_url),
				Some(&c.odoo_db),
				Some(&c.odoo_uid),
				Some(&c.odoo_pw),
				Some(&last_run),
				None,
				false,
				Some(&text),
			)
			.await;

			error!("{}", text);

			return Err(text);
		}
	};

	let s = match str::from_utf8(&a.stdout) {
		Ok(v) => v,
		Err(e) => {
			let text = format!("Invalid UTF-8 sequence on stdout: {}", e);
			error!("{}", text);
			return Err(text);
		}
	};

	trace!("Output:\n{}", &s);

	let er = match str::from_utf8(&a.stderr) {
		Ok(v) => v,
		Err(e) => {
			let text = format!("Invalid UTF-8 sequence on stderr: {}", e);
			error!("{}", text);
			return Err(text);
		}
	};

	if er.is_empty() == false {
		trace!("Errors:\n{}", &er);
	}

	let json = //: Vec<OdooProjectHeader> =
		serde_json::from_str(s); //.expect("JSON was not well-formatted");

	let json = match json {
		Ok(v) => v,
		Err(e) => {
			let _ = write_odoo_call_log(
				&conn,
				"src/modules/odoo/python/odoo_get.py",
				Some(&c.odoo_url),
				Some(&c.odoo_db),
				Some(&c.odoo_uid),
				Some(&c.odoo_pw),
				Some(&last_run),
				None,
				false,
				Some(&format!("JSON was not well-formatted: {}", e)),
			)
			.await;

			return Err(format!("JSON was not well-formatted: {}", e));
		}
	};

	let _ = write_odoo_call_log(
		&conn,
		"src/modules/odoo/python/odoo_get.py",
		Some(&c.odoo_url),
		Some(&c.odoo_db),
		Some(&c.odoo_uid),
		Some(&c.odoo_pw),
		Some(&last_run),
		None,
		true,
		Some(&s),
	)
	.await;

	Ok(json)
}

async fn put_odoo_matches(conn: &PgConnection, matches: Vec<ProjectMatch>) -> Result<(), String> {
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
		Ok(x) => x,
		Err(e) => {
			let _ = write_odoo_call_log(
				&conn,
				"src/modules/odoo/python/odoo_put.py",
				Some(&c.odoo_url),
				Some(&c.odoo_db),
				Some(&c.odoo_uid),
				Some(&c.odoo_pw),
				Some(&odoo_matches),
				None,
				false,
				Some(&format!("Python3 failed: {}", e)),
			)
			.await;

			return Err(format!("Python3 failed: {}", e));
		}
	};

	let _ = write_odoo_call_log(
		&conn,
		"src/modules/odoo/python/odoo_put.py",
		Some(&c.odoo_url),
		Some(&c.odoo_db),
		Some(&c.odoo_uid),
		Some(&c.odoo_pw),
		Some(&odoo_matches),
		None,
		true,
		None,
	)
	.await;

	Ok(())
}

async fn get_odoo_log(conn: &PgConnection) -> Result<Vec<OdooProject>, String> {
	use crate::schema::odoo_projects::dsl::odoo_projects;
	let items = odoo_projects.load::<OdooProject>(conn);

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

	/* iterate log, see what needs update */
	for log1 in &log {
		let h = projects.iter();
		let a = h.filter(|x| x.id == log1.hula_id.to_string()).next();

		if let Some(b) = a {
			let h2 = deals.iter();
			let a2 = h2.filter(|x| x.id == log1.odoo_id).next();

			if let Some(b2) = a2 {
				let updated = update_hula_project_odoo(conn, config, b.id.clone(), b2).await;
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
			let added = insert_hula_project_odoo(conn, config, deal).await;
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
	conn: &PgConnection,
	config: &HulaConfig,
	header: &OdooProjectHeader,
) -> Result<HulaProjectStructureResponse, &'static str> {
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
		Ok(file) => {
			if file.status().as_u16() > 299 {
				let _ = write_hula_log(
					conn,
					None,
					header.id,
					&request_url,
					"POST",
					&format!("{:?}", &data),
					file.status().as_u16().into(),
					&format!("{}", &file.text().await.unwrap()),
				)
				.await;

				return Err("11");
			}
			file
		}
		Err(e) => {
			let _ = write_hula_log(
				conn,
				None,
				header.id,
				&request_url,
				"POST",
				&format!("{:?}", &data),
				0,
				&format!("{}", &e),
			)
			.await;

			return Err("1");
		}
	};

	let status: i32 = response.status().as_u16().into();

	let jiison = response.json().await;

	let jiison2 = match jiison {
		Ok(file) => file,
		Err(e) => {
			let _ = write_hula_log(
				conn,
				None,
				header.id,
				&request_url,
				"POST",
				&format!("{:?}", &data),
				e.status().unwrap().as_u16().into(),
				&format!("{}", &e),
			)
			.await;

			return Err("2");
		}
	};

	let hula_project: HulaProjectStructureResponse = jiison2;

	let _ = write_hula_log(
		conn,
		Some(&hula_project.id.to_string()),
		header.id,
		&request_url,
		"POST",
		&format!("{:?}", &data),
		status,
		&format!("{:?}", &hula_project),
	)
	.await;

	Ok(hula_project)
}

pub async fn update_hula_project_odoo(
	conn: &PgConnection,
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
		Ok(file) => {
			if file.status().as_u16() > 299 {
				let _ = write_hula_log(
					conn,
					Some(&project_id),
					project.id,
					&request_url,
					"PUT",
					&format!("{:?}", &data),
					file.status().as_u16().into(),
					&format!("{}", &file.text().await.unwrap()),
				)
				.await;

				return Err("11");
			}
			file
		}
		Err(e) => {
			let _ = write_hula_log(
				conn,
				Some(&project_id),
				project.id,
				&request_url,
				"PUT",
				&format!("{:?}", &data),
				0,
				&format!("{}", &e),
			)
			.await;

			return Err("1");
		}
	};

	let status: i32 = response.status().as_u16().into();

	let jiison = response.json().await;

	let jiison2 = match jiison {
		Ok(file) => file,
		Err(e) => {
			let _ = write_hula_log(
				conn,
				Some(&project_id),
				project.id,
				&request_url,
				"PUT",
				&format!("{:?}", &data),
				e.status().unwrap().as_u16().into(),
				&format!("{}", &e),
			)
			.await;

			return Err("2");
		}
	};

	let _ = write_hula_log(
		conn,
		Some(&project_id),
		project.id,
		&request_url,
		"PUT",
		&format!("{:?}", &data),
		status,
		&format!("{:?}", &jiison2),
	)
	.await;

	let hula_project: HulaProjectStructureResponse = jiison2;

	Ok(hula_project)
}

async fn write_hula_log(
	conn: &PgConnection,
	hula_id: Option<&str>,
	odoo_id: i32,
	url: &str,
	verb: &str,
	payload: &str,
	status: i32,
	response: &str,
) -> Result<(), &'static str> {
	use crate::schema::hula_call_log::dsl::hula_call_log;

	let hula_id: Option<uuid::Uuid> = match hula_id {
		Some(id) => Some(uuid::Uuid::parse_str(id).expect("uuid parsing failed")),
		None => None,
	};

	let new_log = HulaCallLog {
		id: uuid::Uuid::new_v4(),
		hula_id: hula_id,
		odoo_id: odoo_id,
		url: url.to_string(),
		verb: verb.to_string(),
		payload: payload.to_string(),
		status: status,
		response: response.to_string(),
		updated_by: "hulasync".to_string(),
		updated_at: chrono::Local::now().naive_local(),
	};

	let rows_inserted = diesel::insert_into(hula_call_log)
		.values(&new_log)
		.get_result::<HulaCallLog>(conn);

	let _: Option<&HulaCallLog> = match &rows_inserted {
		Ok(a) => Some(a),
		Err(e) => {
			trace!("ERROR. {:?}", e);
			return Err("failed.");
		}
	};

	return Ok(());
}

async fn write_odoo_call_log(
	conn: &PgConnection,
	script: &str,
	param1: Option<&str>,
	param2: Option<&str>,
	param3: Option<&str>,
	param4: Option<&str>,
	param5: Option<&str>,
	param6: Option<&str>,
	ok: bool,
	response: Option<&str>,
) -> Result<(), &'static str> {
	use crate::schema::odoo_call_log::dsl::odoo_call_log;

	let new_log = OdooCallLog {
		id: uuid::Uuid::new_v4(),
		script: script.to_string(),
		param1: Some(param1.unwrap_or_default().to_string()),
		param2: Some(param2.unwrap_or_default().to_string()),
		param3: Some(param3.unwrap_or_default().to_string()),
		param4: Some(param4.unwrap_or_default().to_string()),
		param5: Some(param5.unwrap_or_default().to_string()),
		param6: Some(param6.unwrap_or_default().to_string()),
		ok: ok,
		response: Some(response.unwrap_or_default().to_string()),
		updated_by: "hulasync".to_string(),
		updated_at: chrono::Local::now().naive_local(),
	};

	let rows_inserted = diesel::insert_into(odoo_call_log)
		.values(&new_log)
		.get_result::<OdooCallLog>(conn);

	let _: Option<&OdooCallLog> = match &rows_inserted {
		Ok(a) => Some(a),
		Err(e) => {
			error!("ERROR. {:?}", e);
			return Err("failed.");
		}
	};

	return Ok(());
}

async fn startup(conn: &PgConnection) -> Result<Option<i64>, &'static str> {
	use crate::schema::hula_call_log::dsl::{hula_call_log, updated_at as hula_updated_at};
	use crate::schema::odoo_call_log::dsl::{
		odoo_call_log, ok, param5, updated_at as odoo_updated_at,
	};

	let discard_limit = chrono::offset::Utc::now().naive_utc() - chrono::Duration::days(7);

	let _ = diesel::delete(odoo_call_log.filter(odoo_updated_at.lt(discard_limit))).execute(conn);
	let _ = diesel::delete(hula_call_log.filter(hula_updated_at.lt(discard_limit))).execute(conn);

	let log = odoo_call_log
		.filter(ok.eq(true))
		.order(odoo_updated_at.desc())
		.first::<OdooCallLog>(conn)
		.optional()
		.unwrap();

	let log_full = odoo_call_log
		.filter(ok.eq(true))
		.filter(param5.eq(""))
		.order(odoo_updated_at.desc())
		.first::<OdooCallLog>(conn)
		.optional()
		.unwrap();

	if let Some(log) = log {
		let mut x = chrono::Datelike::num_days_from_ce(&log.updated_at);
		let y = chrono::Datelike::num_days_from_ce(&chrono::Utc::now().naive_utc());

		if let Some(log_full) = log_full {
			x = chrono::Datelike::num_days_from_ce(&log_full.updated_at);
		}

		if x == y {
			let lag = chrono::Utc::now().naive_utc() - log.updated_at;
			return Ok(Some(lag.num_minutes() + 2));
		}
	}

	Ok(None)
}
