use serde::{Deserialize, Serialize};

/* use crate::errors::ServiceError; */
/*use crate::models::test::{Pool, TestData};*/

#[derive(Deserialize, Serialize, Debug)]
pub struct HulaProject {
	pub id: String,
	pub description: Option<String>,
	pub name: String,
}

#[derive(Deserialize, Serialize, Debug)]
pub struct HulaApiProject {
	pub name: String,
	pub description: Option<String>,
	pub is_hidden: bool,
}

/*
pub struct SyncError {
	pub text: String,
}

impl From<Error> for SyncError {
	fn from(error: Error) -> SyncError {
		SyncError {
			text: format!("{}", &error)
		}
	}
}*/

pub async fn get_hula_projects(config: &HulaConfig) -> Result<Vec<HulaProject>, &'static str> {
	let request_url = format!("{}/api/projects", config.hula_url);
	println!("Calling {}", request_url);

	let client = reqwest::Client::new();

	let response = client
		.get(&request_url)
		.header("Cookie", format!("auth={}", &config.cookie))
		.send()
		.await;

	let response = match response {
		Ok(file) => file,
		Err(e) => {
			println!("{:?}", e);
			return Err("1");
		}
	};

	println!("...Response is: {:?}", &response);

	if response.status() == 204 {
		return Ok(Vec::<HulaProject>::new());
	}

	let jiison = response.json().await;

	let jiison2 = match jiison {
		Ok(file) => file,
		Err(e) => {
			println!("{:?}", e);
			return Err("2");
		}
	};

	let projects: Vec<HulaProject> = jiison2;

	println!("...Got {} projects.", projects.len());

	Ok(projects)
}

pub async fn insert_hula_project(
	config: &HulaConfig,
	name: String,
	description: Option<String>,
) -> Result<String, &'static str> {
	let request_url = format!("{}/api/projects", config.hula_url);
	println!("Calling {}", request_url);

	let project = HulaApiProject {
		name: name,
		description: description,
		is_hidden: false,
	};

	let client = reqwest::Client::new();

	let response = client
		.post(&request_url)
		.header("Cookie", format!("auth={}", config.cookie))
		.json(&project)
		.send()
		.await;

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

	let project: HulaProject = jiison2;

	Ok(project.id)
}

pub async fn update_hula_project(
	config: &HulaConfig,
	project_id: String,
	name: String,
	description: Option<String>,
) -> Result<(), &'static str> {
	let request_url = format!("{}/api/projects/{}", config.hula_url, project_id.clone());
	println!("Calling {}", request_url);

	let project = HulaApiProject {
		description: description,
		is_hidden: false,
		name: name,
	};

	let client = reqwest::Client::new();

	let response = client
		.put(&request_url)
		.header("Cookie", format!("auth={}", config.cookie))
		.json(&project)
		.send()
		.await;

	let response = match response {
		Ok(file) => file,
		Err(e) => {
			println!("{:?}", e);
			return Err("1");
		}
	};

	println!("...Response is: {:?}", &response);

	Ok(())
}

pub struct HulaConfig {
	pub hula_url: String,
	pub cookie: String,
}

#[derive(Deserialize, Serialize, Debug)]
pub struct AuthData {
	pub email: String,
	pub password: String,
}

pub async fn get_config() -> Result<HulaConfig, &'static str> {
	let hula_url = std::env::var("HULA_URL").expect("HULA_URL must be set");
	let hula_uid = std::env::var("HULA_USER_ID").expect("HULA_USER_ID must be set");
	let hula_pwd = std::env::var("HULA_USER_PWD").expect("HULA_USER_PWD must be set");

	let request_url = format!("{}/api/auth", hula_url);
	println!("Calling {}", request_url);

	let data = AuthData {
		email: hula_uid,
		password: hula_pwd,
	};

	let client = reqwest::Client::new();

	let response = client.post(&request_url).json(&data).send().await;

	let response = match response {
		Ok(file) => file,
		Err(e) => {
			println!("{:?}", e);
			return Err("1");
		}
	};

	let cookie = response.cookies().next();
	let cookie = match cookie {
		Some(c) => c,
		None => {
			return Err("11");
		}
	};

	let config = HulaConfig {
		hula_url: std::env::var("HULA_URL").expect("HULA_URL must be set"),
		cookie: cookie.value().to_string(),
	};

	Ok(config)
}

pub async fn close_config(config: &HulaConfig) -> Result<(), &'static str> {
	let request_url = format!("{}/api/auth", config.hula_url);
	println!("Calling {}", request_url);

	let client = reqwest::Client::new();

	let response = client
		.delete(&request_url)
		.header("Cookie", format!("auth={}", config.cookie))
		.send()
		.await;

	let _ = match response {
		Ok(file) => file,
		Err(e) => {
			println!("{:?}", e);
			return Err("1");
		}
	};

	Ok(())
}
