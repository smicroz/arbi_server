use actix_web::{get, post, put, delete, web, HttpResponse, Responder};
use crate::modules::market_pair::market_pair_service::MarketPairService;
use crate::db::mongodb::MongoDbContext;
use crate::modules::market_pair::market_pair_schema::MarketPair;
use mongodb::bson::oid::ObjectId;
use serde::{Deserialize};
use crate::modules::auth::auth_response::ApiResponse;
use tracing::{error};

#[derive(Deserialize)]
struct ObjectIdPath {
    id: String,
}

#[post("/market_pairs")]
pub async fn create_market_pair(market_pair: web::Json<MarketPair>, db_context: web::Data<MongoDbContext>) -> impl Responder {
    match MarketPairService::create_market_pair(market_pair.into_inner(), &db_context).await {
        Ok(market_pair) => HttpResponse::Ok().json(ApiResponse::success("Market pair created successfully", market_pair)),
        Err(err) => {
            error!("Failed to create market pair: {}", err);
            HttpResponse::BadRequest().json(ApiResponse::<String>::error(&err))
        },
    }
}

#[get("/market_pairs/{id}")]
pub async fn get_market_pair(path: web::Path<ObjectIdPath>, db_context: web::Data<MongoDbContext>) -> impl Responder {
    let id = ObjectId::parse_str(&path.id).expect("Invalid ObjectId");
    match MarketPairService::get_market_pair(id, &db_context).await {
        Ok(market_pair) => HttpResponse::Ok().json(ApiResponse::success("Market pair retrieved successfully", market_pair)),
        Err(err) => {
            error!("Failed to retrieve market pair: {}", err);
            HttpResponse::NotFound().json(ApiResponse::<String>::error(&err))
        },
    }
}

#[put("/market_pairs/{id}")]
pub async fn update_market_pair(path: web::Path<ObjectIdPath>, market_pair: web::Json<MarketPair>, db_context: web::Data<MongoDbContext>) -> impl Responder {
    let id = ObjectId::parse_str(&path.id).expect("Invalid ObjectId");
    match MarketPairService::update_market_pair(id, market_pair.into_inner(), &db_context).await {
        Ok(market_pair) => HttpResponse::Ok().json(ApiResponse::success("Market pair updated successfully", market_pair)),
        Err(err) => {
            error!("Failed to update market pair: {}", err);
            HttpResponse::BadRequest().json(ApiResponse::<String>::error(&err))
        },
    }
}

#[delete("/market_pairs/{id}")]
pub async fn delete_market_pair(path: web::Path<ObjectIdPath>, db_context: web::Data<MongoDbContext>) -> impl Responder {
    let id = ObjectId::parse_str(&path.id).expect("Invalid ObjectId");
    match MarketPairService::delete_market_pair(id, &db_context).await {
        Ok(_) => HttpResponse::Ok().json(ApiResponse::success("Market pair deleted successfully", ())),
        Err(err) => {
            error!("Failed to delete market pair: {}", err);
            HttpResponse::BadRequest().json(ApiResponse::<String>::error(&err))
        },
    }
}

#[get("/market_pairs")]
pub async fn get_all_market_pairs(db_context: web::Data<MongoDbContext>) -> impl Responder {
    match MarketPairService::get_all_market_pairs(&db_context).await {
        Ok(market_pairs) => HttpResponse::Ok().json(ApiResponse::success("Market pairs retrieved successfully", market_pairs)),
        Err(err) => {
            error!("Failed to retrieve market pairs: {}", err);
            HttpResponse::BadRequest().json(ApiResponse::<String>::error(&err))
        },
    }
}
