use actix_web::{get, post, put, delete, web, HttpResponse, Responder};
use crate::modules::exchange::exchange_service::ExchangeService;
use crate::db::mongodb::MongoDbContext;
use crate::modules::exchange::exchange_schema::Exchange;
use mongodb::bson::oid::ObjectId;
use serde::{Deserialize};
use crate::modules::auth::auth_response::ApiResponse;
use tracing::{ error};

#[derive(Deserialize)]
struct ObjectIdPath {
    id: String,
}

#[post("/exchanges")]
pub async fn create_exchange(exchange: web::Json<Exchange>, db_context: web::Data<MongoDbContext>) -> impl Responder {
    match ExchangeService::create_exchange(exchange.into_inner(), &db_context).await {
        Ok(exchange) => HttpResponse::Ok().json(ApiResponse::success("Exchange created successfully", exchange)),
        Err(err) => {
            error!("Failed to create exchange: {}", err);
            HttpResponse::BadRequest().json(ApiResponse::<String>::error(&err))
        },
    }
}

#[get("/exchanges/{id}")]
pub async fn get_exchange(path: web::Path<ObjectIdPath>, db_context: web::Data<MongoDbContext>) -> impl Responder {
    let id = ObjectId::parse_str(&path.id).expect("Invalid ObjectId");
    match ExchangeService::get_exchange(id, &db_context).await {
        Ok(exchange) => HttpResponse::Ok().json(ApiResponse::success("Exchange retrieved successfully", exchange)),
        Err(err) => {
            error!("Failed to retrieve exchange: {}", err);
            HttpResponse::NotFound().json(ApiResponse::<String>::error(&err))
        },
    }
}

#[put("/exchanges/{id}")]
pub async fn update_exchange(path: web::Path<ObjectIdPath>, exchange: web::Json<Exchange>, db_context: web::Data<MongoDbContext>) -> impl Responder {
    let id = ObjectId::parse_str(&path.id).expect("Invalid ObjectId");
    match ExchangeService::update_exchange(id, exchange.into_inner(), &db_context).await {
        Ok(exchange) => HttpResponse::Ok().json(ApiResponse::success("Exchange updated successfully", exchange)),
        Err(err) => {
            error!("Failed to update exchange: {}", err);
            HttpResponse::BadRequest().json(ApiResponse::<String>::error(&err))
        },
    }
}

#[delete("/exchanges/{id}")]
pub async fn delete_exchange(path: web::Path<ObjectIdPath>, db_context: web::Data<MongoDbContext>) -> impl Responder {
    let id = ObjectId::parse_str(&path.id).expect("Invalid ObjectId");
    match ExchangeService::delete_exchange(id, &db_context).await {
        Ok(_) => HttpResponse::Ok().json(ApiResponse::success("Exchange deleted successfully", ())),
        Err(err) => {
            error!("Failed to delete exchange: {}", err);
            HttpResponse::BadRequest().json(ApiResponse::<String>::error(&err))
        },
    }
}

#[get("/exchanges")]
pub async fn get_all_exchanges(db_context: web::Data<MongoDbContext>) -> impl Responder {
    match ExchangeService::get_all_exchanges(&db_context).await {
        Ok(exchanges) => HttpResponse::Ok().json(ApiResponse::success("Exchanges retrieved successfully", exchanges)),
        Err(err) => {
            error!("Failed to retrieve exchanges: {}", err);
            HttpResponse::BadRequest().json(ApiResponse::<String>::error(&err))
        },
    }
}
