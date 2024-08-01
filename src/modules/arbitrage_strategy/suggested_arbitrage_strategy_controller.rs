use actix_web::{get, web, HttpResponse, Responder};
use serde::{Deserialize, Serialize};
use crate::modules::auth::auth_response::ApiResponse;
use crate::db::mongodb::MongoDbContext;
use crate::modules::arbitrage_strategy::suggested_arbitrage_strategy_service::SuggestedArbitrageStrategyService;
use crate::modules::arbitrage_strategy::arbitrage_strategy_schema::{ArbitrageStrategy, ArbitrageType};
use mongodb::bson::oid::ObjectId;  // Añadimos esta importación

#[derive(Deserialize)]
struct SuggestedStrategyQuery {
    exchange1: String,
    exchange2: String,
    strategy_type: ArbitrageType,
}

#[derive(Serialize)]
struct SuggestedStrategyResponse {
    strategies: Vec<ArbitrageStrategy>,
}

#[get("/arbitrage-strategies/suggested")]
pub async fn get_suggested_strategies(
    db_context: web::Data<MongoDbContext>,
    query: web::Query<SuggestedStrategyQuery>,
) -> impl Responder {
    let exchange1 = match ObjectId::parse_str(&query.exchange1) {
        Ok(id) => id,
        Err(_) => return HttpResponse::BadRequest().json(ApiResponse::<String>::error("Invalid exchange1 ID")),
    };
    let exchange2 = match ObjectId::parse_str(&query.exchange2) {
        Ok(id) => id,
        Err(_) => return HttpResponse::BadRequest().json(ApiResponse::<String>::error("Invalid exchange2 ID")),
    };

    match SuggestedArbitrageStrategyService::get_suggested_strategies(
        &db_context,
        exchange1,
        exchange2,
        query.strategy_type.clone(),
    ).await {
        Ok(strategies) => HttpResponse::Ok().json(ApiResponse::success("Suggested strategies retrieved successfully", SuggestedStrategyResponse { strategies })),
        Err(err) => HttpResponse::BadRequest().json(ApiResponse::<String>::error(&err)),
    }
}