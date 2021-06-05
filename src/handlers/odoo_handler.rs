use actix_web::{/*error::BlockingError, */web, HttpResponse};

/*use crate::errors::ServiceError; */
use crate::models::odoo_project::{Pool/*, TestData*/};
use crate::modules::odoo::odoo_module::do_process;

#[tokio::main]
pub async fn get_test(
	pool: web::Data<Pool>,
) -> HttpResponse {
	println!("Henlo world");

	let _result = do_process(pool).await;

	HttpResponse::Ok().finish()
}
