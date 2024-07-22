use actix_web::{get, post, put, delete, web, HttpResponse, Responder};
use crate::modules::asset::asset_service::AssetService;
use crate::db::mongodb::MongoDbContext;
use crate::modules::asset::asset_schema::Asset;
use mongodb::bson::oid::ObjectId;
use serde::{Deserialize};
use serde_json::json; 

#[derive(Deserialize)]
struct ObjectIdPath {
    id: String,
}

#[derive(Deserialize)]
struct AssetQuery {
    page: Option<u64>,
    per_page: Option<u64>,
    include_exchange: Option<bool>,
    search: Option<String>,
}

#[post("/assets")]
pub async fn create_asset(asset: web::Json<Asset>, db_context: web::Data<MongoDbContext>) -> impl Responder {
    match AssetService::create_asset(asset.into_inner(), &db_context).await {
        Ok(asset) => HttpResponse::Ok().json(asset),
        Err(err) => HttpResponse::BadRequest().body(err),
    }
}

#[get("/assets/{id}")]
pub async fn get_asset(path: web::Path<ObjectIdPath>, db_context: web::Data<MongoDbContext>) -> impl Responder {
    let id = ObjectId::parse_str(&path.id).expect("Invalid ObjectId");
    match AssetService::get_asset(id, &db_context).await {
        Ok(asset) => HttpResponse::Ok().json(asset),
        Err(err) => HttpResponse::NotFound().body(err),
    }
}

#[put("/assets/{id}")]
pub async fn update_asset(path: web::Path<ObjectIdPath>, asset: web::Json<Asset>, db_context: web::Data<MongoDbContext>) -> impl Responder {
    let id = ObjectId::parse_str(&path.id).expect("Invalid ObjectId");
    match AssetService::update_asset(id, asset.into_inner(), &db_context).await {
        Ok(asset) => HttpResponse::Ok().json(asset),
        Err(err) => HttpResponse::BadRequest().body(err),
    }
}

#[delete("/assets/{id}")]
pub async fn delete_asset(path: web::Path<ObjectIdPath>, db_context: web::Data<MongoDbContext>) -> impl Responder {
    let id = ObjectId::parse_str(&path.id).expect("Invalid ObjectId");
    match AssetService::delete_asset(id, &db_context).await {
        Ok(_) => HttpResponse::Ok().finish(),
        Err(err) => HttpResponse::BadRequest().body(err),
    }
}

#[get("/assets")]
pub async fn get_all_assets(
    db_context: web::Data<MongoDbContext>,
    query: web::Query<AssetQuery>,
) -> impl Responder {
    let page = query.page.unwrap_or(1);
    let per_page = query.per_page.unwrap_or(20);
    let include_exchange = query.include_exchange.unwrap_or(false);
    let search = query.search.clone();

    match AssetService::get_all_assets(&db_context, page, per_page, include_exchange, search).await {
        Ok((assets, total)) => HttpResponse::Ok().json(json!({
            "assets": assets,
            "total": total,
            "page": page,
            "per_page": per_page
        })),
        Err(err) => HttpResponse::BadRequest().body(err),
    }
}