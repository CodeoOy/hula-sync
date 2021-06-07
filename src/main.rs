#[macro_use]
extern crate diesel;

use diesel::prelude::*;
use diesel::r2d2::{self, ConnectionManager};

mod models;
mod schema;
mod modules;
mod hulautils;
mod background;

#[tokio::main]
async fn main() -> std::io::Result<()> {
	dotenv::dotenv().ok();
	std::env::set_var(
		"RUST_LOG",
		"hula-sync=debug",
	);
	env_logger::init();
	let database_url = std::env::var("DATABASE_URL").expect("DATABASE_URL must be set");

	// create db connection pool
	let manager = ConnectionManager::<PgConnection>::new(database_url.to_string());
	let pool: models::odoo_project::Pool = r2d2::Pool::builder()
		.build(manager)
		.expect("Failed to create pool.");

	Ok(background::start_background(pool.clone()).await)
}
