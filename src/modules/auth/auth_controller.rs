use actix_web::{post, web, HttpResponse, Responder};
use crate::modules::auth::auth_service::AuthService;
use crate::db::mongodb::MongoDbContext;
use crate::modules::auth::auth_model::{RegisterRequest, LoginRequest};
use crate::modules::auth::auth_response::ApiResponse;

#[post("/register")]
pub async fn register(data: web::Json<RegisterRequest>, db_context: web::Data<MongoDbContext>) -> impl Responder {
    let request = data.into_inner();
    match AuthService::register(&request.name, &request.email, &request.password, &db_context).await {
        Ok(auth_response) => HttpResponse::Ok().json(ApiResponse::success("Registration successful", auth_response)),
        Err(err) => HttpResponse::BadRequest().json(ApiResponse::<()>::error(&err)),
    }
}

#[post("/login")]
pub async fn login(data: web::Json<LoginRequest>, db_context: web::Data<MongoDbContext>) -> impl Responder {
    let request = data.into_inner();
    match AuthService::login(&request.email, &request.password, &db_context).await {
        Ok(auth_response) => HttpResponse::Ok().json(ApiResponse::success("Login successful", auth_response)),
        Err(err) => HttpResponse::Unauthorized().json(ApiResponse::<()>::error(&err)),
    }
}
