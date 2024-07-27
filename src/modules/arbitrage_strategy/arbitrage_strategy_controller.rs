use actix_web::{get, post, put, delete, web, HttpResponse, Responder};
use crate::modules::arbitrage_strategy::arbitrage_strategy_service::ArbitrageStrategyService;
use crate::db::mongodb::MongoDbContext;
use crate::modules::arbitrage_strategy::arbitrage_strategy_schema::{ArbitrageStrategy, ArbitrageType};
use mongodb::bson::oid::ObjectId;
use serde::{Deserialize};
use crate::modules::auth::auth_response::ApiResponse;
use tracing::error;
use serde_json::json;

#[derive(Deserialize)]
struct ObjectIdPath {
    id: String,
}

#[derive(Deserialize)]
struct ArbitrageStrategyQuery {
    page: Option<u64>,
    per_page: Option<u64>,
    arbitrage_type: Option<ArbitrageType>,
}

#[post("/arbitrage-strategies")]
pub async fn create_arbitrage_strategy(strategy: web::Json<ArbitrageStrategy>, db_context: web::Data<MongoDbContext>) -> impl Responder {
    match ArbitrageStrategyService::create_arbitrage_strategy(strategy.into_inner(), &db_context).await {
        Ok(strategy) => HttpResponse::Ok().json(ApiResponse::success("Arbitrage strategy created successfully", strategy)),
        Err(err) => {
            error!("Failed to create arbitrage strategy: {}", err);
            HttpResponse::BadRequest().json(ApiResponse::<String>::error(&err))
        },
    }
}

#[get("/arbitrage-strategies/{id}")]
pub async fn get_arbitrage_strategy(path: web::Path<ObjectIdPath>, db_context: web::Data<MongoDbContext>) -> impl Responder {
    let id = ObjectId::parse_str(&path.id).expect("Invalid ObjectId");
    match ArbitrageStrategyService::get_arbitrage_strategy(id, &db_context).await {
        Ok(strategy) => HttpResponse::Ok().json(ApiResponse::success("Arbitrage strategy retrieved successfully", strategy)),
        Err(err) => {
            error!("Failed to retrieve arbitrage strategy: {}", err);
            HttpResponse::NotFound().json(ApiResponse::<String>::error(&err))
        },
    }
}

#[put("/arbitrage-strategies/{id}")]
pub async fn update_arbitrage_strategy(path: web::Path<ObjectIdPath>, strategy: web::Json<ArbitrageStrategy>, db_context: web::Data<MongoDbContext>) -> impl Responder {
    let id = ObjectId::parse_str(&path.id).expect("Invalid ObjectId");
    match ArbitrageStrategyService::update_arbitrage_strategy(id, strategy.into_inner(), &db_context).await {
        Ok(strategy) => HttpResponse::Ok().json(ApiResponse::success("Arbitrage strategy updated successfully", strategy)),
        Err(err) => {
            error!("Failed to update arbitrage strategy: {}", err);
            HttpResponse::BadRequest().json(ApiResponse::<String>::error(&err))
        },
    }
}

#[delete("/arbitrage-strategies/{id}")]
pub async fn delete_arbitrage_strategy(path: web::Path<ObjectIdPath>, db_context: web::Data<MongoDbContext>) -> impl Responder {
    let id = ObjectId::parse_str(&path.id).expect("Invalid ObjectId");
    match ArbitrageStrategyService::delete_arbitrage_strategy(id, &db_context).await {
        Ok(_) => HttpResponse::Ok().json(ApiResponse::success("Arbitrage strategy deleted successfully", ())),
        Err(err) => {
            error!("Failed to delete arbitrage strategy: {}", err);
            HttpResponse::BadRequest().json(ApiResponse::<String>::error(&err))
        },
    }
}

#[get("/arbitrage-strategies")]
pub async fn get_all_arbitrage_strategies(
    db_context: web::Data<MongoDbContext>,
    query: web::Query<ArbitrageStrategyQuery>,
) -> impl Responder {
    let page = query.page.unwrap_or(1);
    let per_page = query.per_page.unwrap_or(20);

    println!("message");
    match ArbitrageStrategyService::get_all_arbitrage_strategies(
        &db_context,
        page,
        per_page,
        query.arbitrage_type.clone(),
    ).await {
        Ok((strategies, total)) => HttpResponse::Ok().json(ApiResponse::success("Arbitrage strategies retrieved successfully", json!({
            "strategies": strategies,
            "total": total,
            "page": page,
            "per_page": per_page
        }))),
        Err(err) => {
            error!("Failed to retrieve arbitrage strategies: {}", err);
            HttpResponse::BadRequest().json(ApiResponse::<String>::error(&err))
        },
    }
}