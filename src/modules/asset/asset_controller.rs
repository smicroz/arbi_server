use actix_web::{get, post, put, delete, web, HttpResponse, Responder};
use crate::modules::asset::asset_service::AssetService;
use crate::db::mongodb::MongoDbContext;
use crate::modules::asset::asset_schema::Asset;
use mongodb::bson::oid::ObjectId;
use serde::{Deserialize};

#[derive(Deserialize)]
struct ObjectIdPath {
    id: String,
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
pub async fn get_all_assets(db_context: web::Data<MongoDbContext>) -> impl Responder {
    match AssetService::get_all_assets(&db_context).await {
        Ok(assets) => HttpResponse::Ok().json(assets),
        Err(err) => HttpResponse::BadRequest().body(err),
    }
}
