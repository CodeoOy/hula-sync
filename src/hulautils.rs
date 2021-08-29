use serde::{Deserialize, Serialize};

/* use crate::errors::ServiceError; */
/*use crate::models::test::{Pool, TestData};*/

#[derive(Deserialize, Serialize, Debug)]
pub struct HulaProject {
	pub id: String,
	pub name: String,
}

#[derive(Deserialize, Serialize, Debug)]
pub struct HulaApiProject {
	pub name: String,
	pub is_hidden: bool,
}

pub async fn get_hula_projects() -> Result<Vec<HulaProject>, &'static str> {
	let hula_url = std::env::var("HULA_URL").expect("HULA_URL must be set");

	let hula_key = std::env::var("HULA_API_KEY").expect("HULA_API_KEY must be set");

	let request_url = format!("{}/api/projects", hula_url);
	println!("Calling {}", request_url);

	let client = reqwest::Client::new();

	let response = client
		.get(&request_url)
		.header("Cookie", format!("auth={}", hula_key))
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

pub async fn insert_hula_project(name: String) -> Result<String, &'static str> {
	let hula_url = std::env::var("HULA_URL").expect("HULA_URL must be set");

	let hula_key = std::env::var("HULA_API_KEY").expect("HULA_API_KEY must be set");

	let request_url = format!("{}/api/projects", hula_url);
	println!("Calling {}", request_url);

	let project = HulaApiProject {
		name: name,
		is_hidden: false,
	};

	let client = reqwest::Client::new();

	let response = client
		.post(&request_url)
		.header("Cookie", format!("auth={}", hula_key))
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

pub async fn update_hula_project(project_id: String, name: String) -> Result<(), &'static str> {
	let hula_url = std::env::var("HULA_URL").expect("HULA_URL must be set");

	let hula_key = std::env::var("HULA_API_KEY").expect("HULA_API_KEY must be set");

	let request_url = format!("{}/api/projects/{}", hula_url, project_id.clone());
	println!("Calling {}", request_url);

	let project = HulaProject {
		id: project_id,
		name: name,
	};

	let client = reqwest::Client::new();

	let response = client
		.put(&request_url)
		.header("Cookie", format!("auth={}", hula_key))
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
	pub hula_key: String,
}

pub fn get_config() -> HulaConfig {
	let config = HulaConfig {
		hula_url: std::env::var("HULA_URL").expect("HULA_URL must be set"),
		hula_key: std::env::var("HULA_API_KEY").expect("HULA_API_KEY must be set"),
	};

	config
}
