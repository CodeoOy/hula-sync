use actix_web::{error::BlockingError, web, HttpResponse};
use diesel::{prelude::*, PgConnection};
use serde::{Serialize, Deserialize};

use crate::errors::ServiceError;
use crate::models::test::{Pool, TestData};

pub async fn get_test(
	pool: web::Data<Pool>,
) -> HttpResponse {
	println!("Henlo world");
	HttpResponse::Ok().finish()
}