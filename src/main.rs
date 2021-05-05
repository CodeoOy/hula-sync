#[macro_use]
extern crate diesel;

use actix_web::http::{header, Method, StatusCode};
use actix_web::{get, middleware, web, App, HttpRequest, HttpResponse, HttpServer, Result};
use diesel::prelude::*;
use diesel::r2d2::{self, ConnectionManager};

mod models;
mod handlers;

#[actix_rt::main]
async fn main() -> std::io::Result<()> {
	dotenv::dotenv().ok();
	std::env::set_var(
		"RUST_LOG",
		"hula-sync=debug,actix_web=info,actix_server=info",
	);
	env_logger::init();
	let database_url = std::env::var("DATABASE_URL").expect("DATABASE_URL must be set");

	// create db connection pool
	let manager = ConnectionManager::<PgConnection>::new(database_url);
	let pool: models::test::Pool = r2d2::Pool::builder()
		.build(manager)
		.expect("Failed to create pool.");
	let domain: String = std::env::var("DOMAIN").unwrap_or_else(|_| "localhost".to_string());

	// Start http server
	HttpServer::new(move || {
	App::new()
			.data(pool.clone())
			// enable logger
			.wrap(middleware::Logger::default())
			.data(web::JsonConfig::default().limit(4096))
			// everything under '/api/' route
			.service(
				web::scope("/api")
					.service(
						web::resource("/test")
						.route(web::get().to(handlers::test::get_test))
					)
			)
			.service(web::resource("/").route(web::get().to(|req: HttpRequest| {
				println!("HTTP REQ:\n{:?}\n", req);
				HttpResponse::Found()
					.header(header::LOCATION, "index.html")
					.finish()
			})))
	})
	.bind("localhost:8088")?
	.run()
	.await
}
