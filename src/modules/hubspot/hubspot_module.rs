use actix_web::{/*error::BlockingError, */web/*, HttpResponse*/};
/*use diesel::{prelude::*, PgConnection};*/
use serde::{Deserialize};

use crate::models::odoo_project::{Pool};
use crate::hulautils::get_hula_projects;

use std::str;

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
}

#[derive(Deserialize, Debug)]
pub struct HubspotDealName {
    value: String,
}

#[derive(Deserialize, Debug)]
struct HulaProject {
    id: String,
    name: String,
}

/*
#[tokio::main]
pub async fn process(
	pool: web::Data<Pool>
) {
	let result = do_process(pool);
}
*/

pub async fn do_process(
	_pool: web::Data<Pool>,
) -> Result<(), String> {
	println!("Henlo world");

	let _hubspot_deals = get_hubspot_deals().await;
	println!("hubspot gotten");

	let _hula_projects = get_hula_projects().await;
	println!("hula gotten");

	Ok(())
}




pub async fn get_hubspot_deals(
) -> Result<HubspotHeader, &'static str> {

	let hubspot_key =
		std::env::var("HUBSPOT_API_KEY").expect("HUBSPOT_API_KEY must be set");

    let request_url = format!("https://api.hubapi.com/deals/v1/deal/paged?hapikey={}&properties=dealname",
		hubspot_key);
		
    println!("Calling {}", request_url);

	let response = reqwest::get(&request_url).await;
	
	let response = match response {
        Ok(file) => file,
        Err(e) => {
			println!("{:?}", e);
			return Err("1");
		},
    };

	// println!("...Response is: {:?}", &response);
	
    let jiison = response.json().await;

	let jiison2 = match jiison {
        Ok(file) => file,
        Err(e) => {
			println!("{:?}", e);
			return Err("2");
		},
    };

	let header: HubspotHeader = jiison2;

	println!("...Got {}", header.deals.len());

	Ok(header)
}






/*


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

	let mut a = Command::new("python3")
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
*/


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