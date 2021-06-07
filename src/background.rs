use diesel::prelude::*;
use std::time::Duration;
use async_std::task;

use crate::modules::odoo::odoo_module;
use crate::modules::hubspot::hubspot_module;
use crate::models::odoo_project::{Pool/*, TestData*/};

pub async fn start_background(
	pool: Pool,
) {
	println!("Howdy");

	let modules = std::env::var("MODULES").expect("MODULES must be set");
	let m: Vec<&str> = modules.split(",").collect();

	let sleep = std::env::var("SLEEP").expect("SLEEP must be set");
	let seconds: u64 = sleep.parse().unwrap_or(60);

	loop {
		println!("Howdy loopy");

		let conn: &PgConnection = &pool.get().unwrap();

		let a = m.iter();

		for s in a {
			match *s {
				"odoo" => {
					let _ = odoo_module::do_process(conn).await;
				},
				"hubspot" => {
					let _ = hubspot_module::do_process(conn).await;
				},
				_ => println!("Unknown module defined in MODULES variable!"),
			};
		}
	
		task::sleep(Duration::from_secs(seconds)).await;
	}
}
