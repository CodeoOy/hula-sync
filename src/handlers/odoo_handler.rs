use actix_web::{error::BlockingError, web, HttpResponse};
use diesel::{prelude::*, PgConnection};
use serde::{Serialize, Deserialize};

use crate::errors::ServiceError;
use crate::models::test::{Pool, TestData};

use reqwest::{Error, Response};

use std::iter::Iterator;

use std::process::Command;

use tokio;


extern crate xmlrpc;
use xmlrpc::{Request, Value};

use std::thread;

use std::fmt;
use std::str;



#[derive(Deserialize, Debug)]
struct Header {
    deals: Vec<Deal>,
}

#[derive(Deserialize, Debug)]
struct Deal {
    dealId: u64,
	properties: Properties,
}

#[derive(Deserialize, Debug)]
struct Properties {
    dealname: DealName,
}

#[derive(Deserialize, Debug)]
struct DealName {
    value: String,
}

#[derive(Deserialize, Debug)]
struct HulaProject {
    id: String,
    name: String,
}

#[tokio::main]
async fn do_it() {
	// The Python example server exports Python's `pow` method. Let's call it!
    let pow_request = Request::new("start"); //.arg(2).arg(8); // Compute 2**8

    let request_result = pow_request.call_url("https://demo.odoo.com/start");

    println!("Result: {:?}", request_result);

}


#[tokio::main]
pub async fn get_test(
	pool: web::Data<Pool>,
) -> HttpResponse {
	println!("Henlo world");

	let mut a = Command::new("python3")
        .arg("/home/pekka/test/test1.py")
        //.arg("-a")
        //.spawn()
		.output()
        .expect("ls command failed to start");
	
		let s = match str::from_utf8(&a.stdout) {
			Ok(v) => v,
			Err(e) => panic!("Invalid UTF-8 sequence: {}", e),
		};

	println!("Henlo world2");
	println!("{}", s);


/*
		thread::spawn(|| {
        do_it();
    }).join().expect("Thread panicked");
*/
    //let pow_result = request_result.unwrap();
    //assert_eq!(pow_result, Value::Int(2i32.pow(8)));




/*	let my = get_HubSpotHeader().await;

	let mut my = match my {
        Ok(file) => file,
        Err(e) => {
			println!("{:?}", e);
			return HttpResponse::Ok().finish();
		},
    };

	let my2 = get_HulaProjects().await;

	let mut my2 = match my2 {
        Ok(file) => file,
        Err(e) => {
			println!("{:?}", e);
			return HttpResponse::Ok().finish();
		},
    };

	update_HulaProjects(my, my2).await;
*/
	HttpResponse::Ok().finish()
}

pub async fn get_HubSpotHeader(
) -> Result<Header, &'static str> {

	let hubspot_key =
		std::env::var("HUBSPOT_API_KEY").expect("HUBSPOT_API_KEY must be set");

    let request_url = format!("https://api.hubapi.com/deals/v1/deal/paged?hapikey={}&properties=dealname",
		hubspot_key);
		
    println!("Calling {}", request_url);

	let response = reqwest::get(&request_url).await;
	
	let mut response = match response {
        Ok(file) => file,
        Err(e) => {
			println!("{:?}", e);
			return Err("1");
		},
    };

    let jiison = response.json().await;

	let mut jiison2 = match jiison {
        Ok(file) => file,
        Err(e) => {
			println!("{:?}", e);
			return Err("2");
		},
    };

	let mut header: Header = jiison2;

	println!("...Got {}", header.deals.len());

	Ok(header)
}

pub async fn get_HulaProjects(
) -> Result<Vec<HulaProject>, &'static str> {
    let request_url = format!("http://localhost:8086/api/projects");
    println!("Calling {}", request_url);

	let response = reqwest::get(&request_url).await;
	
	let mut response = match response {
        Ok(file) => file,
        Err(e) => {
			println!("{:?}", e);
			return Err("1");
		},
    };

    let jiison = response.json().await;

	let mut jiison2 = match jiison {
        Ok(file) => file,
        Err(e) => {
			println!("{:?}", e);
			return Err("2");
		},
    };

	let mut projects: Vec<HulaProject> = jiison2;

	println!("...Got {}", projects.len());

	Ok(projects)
}

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

