use actix_web::{get, post, put, delete, web, HttpResponse, Responder};
use crate::modules::market_pair::market_pair_service::MarketPairService;
use crate::db::mongodb::MongoDbContext;
use crate::modules::market_pair::market_pair_schema::MarketPair;
use mongodb::bson::oid::ObjectId;
use serde::{Deserialize};
use crate::modules::auth::auth_response::ApiResponse;
use tracing::{error};
use serde_json::json;

#[derive(Deserialize)]
struct ObjectIdPath {
    pub id: String,
}

#[derive(Deserialize)]
struct MarketPairQuery {
    page: Option<u64>,
    per_page: Option<u64>,
    exchange_id: Option<String>,
    search: Option<String>,
}

#[get("/conversion_pairs")]
pub async fn get_conversion_pairs(
    query: web::Query<ConversionPairsQuery>,
    db_context: web::Data<MongoDbContext>
) -> impl Responder {
    let pair1 = match ObjectId::parse_str(&query.pair1) {
        Ok(id) => id,
        Err(_) => return HttpResponse::BadRequest().json(ApiResponse::<String>::error("Invalid pair1 ID")),
    };
    let pair2 = match ObjectId::parse_str(&query.pair2) {
        Ok(id) => id,
        Err(_) => return HttpResponse::BadRequest().json(ApiResponse::<String>::error("Invalid pair2 ID")),
    };

    match MarketPairService::get_conversion_pairs(&db_context, pair1, pair2).await {
        Ok(pairs) => HttpResponse::Ok().json(ApiResponse::success("Conversion pairs retrieved successfully", pairs)),
        Err(err) => {
            error!("Failed to retrieve conversion pairs: {}", err);
            HttpResponse::BadRequest().json(ApiResponse::<String>::error(&err))
        },
    }
}

#[derive(Deserialize)]
struct ConversionPairsQuery {
    pair1: String,
    pair2: String,
}

#[get("/market_pairs/by_exchange/{exchange_id}")]
pub async fn get_market_pairs_by_exchange(
    exchange_id: web::Path<String>,
    db_context: web::Data<MongoDbContext>
) -> impl Responder {
    match ObjectId::parse_str(&*exchange_id) {
        Ok(oid) => {
            match MarketPairService::get_all_market_pairs_by_exchange(&db_context, oid).await {
                Ok(market_pairs) => HttpResponse::Ok().json(ApiResponse::success("Market pairs retrieved successfully", market_pairs)),
                Err(err) => {
                    error!("Failed to retrieve market pairs by exchange: {}", err);
                    HttpResponse::BadRequest().json(ApiResponse::<String>::error(&err.to_string()))
                },
            }
        },
        Err(_) => {
            HttpResponse::BadRequest().json(ApiResponse::<String>::error("Invalid exchange ID"))
        }
    }
}

#[get("/market_pairs/with_pagination")]
pub async fn get_all_market_pairs_with_pagination(
    db_context: web::Data<MongoDbContext>,
    query: web::Query<MarketPairQuery>,
) -> impl Responder {
    let page = query.page.unwrap_or(1);
    let per_page = query.per_page.unwrap_or(20);

    match MarketPairService::get_all_market_pairs_with_pagination(
        &db_context,
        page,
        per_page,
        query.exchange_id.clone(),
        query.search.clone(),
    ).await {
        Ok((market_pairs, total)) => HttpResponse::Ok().json(ApiResponse::success("Market pairs retrieved successfully", json!({
            "market_pairs": market_pairs,
            "total": total,
            "page": page,
            "per_page": per_page
        }))),
        Err(err) => {
            error!("Failed to retrieve market pairs in GET /market_pairs/with_pagination: {}", err);
            HttpResponse::BadRequest().json(ApiResponse::<String>::error(&err))
        },
    }
}


#[post("/market_pairs")]
pub async fn create_market_pair(market_pair: web::Json<MarketPair>, db_context: web::Data<MongoDbContext>) -> impl Responder {
    match MarketPairService::create_market_pair(market_pair.into_inner(), &db_context).await {
        Ok(market_pair) => HttpResponse::Ok().json(ApiResponse::success("Market pair created successfully", market_pair)),
        Err(err) => {
            error!("Failed to create market pair in : {}", err);
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


#[get("/market_pairs/conversion_pairs_for_arbitrage")]
pub async fn get_conversion_pairs_for_arbitrage(
    query: web::Query<ConversionPairsQueryToArbitrage>,
    db_context: web::Data<MongoDbContext>
) -> impl Responder {
    // Use query.pair1 and query.pair2 here
    // Don't try to parse these as ObjectIds
    match MarketPairService::get_conversion_pairs_for_arbitrage(&db_context, &query.quote_asset1, &query.quote_asset2).await {
        Ok(pairs) => HttpResponse::Ok().json(ApiResponse::success("Conversion pairs for arbitrage retrieved successfully", pairs)),
        Err(err) => {
            error!("Failed to retrieve conversion pairs for arbitrage: {}", err);
            HttpResponse::BadRequest().json(ApiResponse::<String>::error(&err))
        },
    }
}

#[derive(Deserialize)]
struct ConversionPairsQueryToArbitrage {
    quote_asset1: String,
    quote_asset2: String,
}




