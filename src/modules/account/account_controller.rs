use actix_web::{get, put, web, HttpResponse, Responder};
use crate::modules::account::account_service::{AccountService, UpdateUserRequest};
use crate::db::mongodb::MongoDbContext;
use crate::modules::auth::auth_response::ApiResponse;
use mongodb::bson::oid::ObjectId;

#[get("/account/{id}")]
pub async fn get_user(path: web::Path<String>, db_context: web::Data<MongoDbContext>) -> impl Responder {
    let user_id = path.into_inner();
    let user_id = match ObjectId::parse_str(&user_id) {
        Ok(id) => id,
        Err(_) => return HttpResponse::BadRequest().json(ApiResponse::<()>::error("Invalid user ID")),
    };

    match AccountService::get_user(user_id, &db_context).await {
        Ok(user) => HttpResponse::Ok().json(ApiResponse::success("User retrieved successfully", user)),
        Err(err) => HttpResponse::BadRequest().json(ApiResponse::<()>::error(&err)),
    }
}

#[put("/account/{id}")]
pub async fn update_user(path: web::Path<String>, data: web::Json<UpdateUserRequest>, db_context: web::Data<MongoDbContext>) -> impl Responder {
    let user_id = path.into_inner();
    let user_id = match ObjectId::parse_str(&user_id) {
        Ok(id) => id,
        Err(_) => return HttpResponse::BadRequest().json(ApiResponse::<()>::error("Invalid user ID")),
    };

    match AccountService::update_user(user_id, data.into_inner(), &db_context).await {
        Ok(user) => HttpResponse::Ok().json(ApiResponse::success("User updated successfully", user)),
        Err(err) => HttpResponse::BadRequest().json(ApiResponse::<()>::error(&err)),
    }
}
