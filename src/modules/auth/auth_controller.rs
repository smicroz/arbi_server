use actix_web::{post, web, HttpResponse, Responder};
use crate::modules::auth::auth_service::AuthService;
use crate::db::mongodb::MongoDbContext;
use crate::modules::auth::auth_model::{RegisterRequest, LoginRequest};

#[post("/register")]
pub async fn register(data: web::Json<RegisterRequest>, db_context: web::Data<MongoDbContext>) -> impl Responder {
    let request = data.into_inner();
    match AuthService::register(&request.name, &request.email, &request.password, &db_context).await {
        Ok(user) => HttpResponse::Ok().json(user),
        Err(err) => HttpResponse::BadRequest().body(err),
    }
}

#[post("/login")]
pub async fn login(data: web::Json<LoginRequest>, db_context: web::Data<MongoDbContext>) -> impl Responder {
    let request = data.into_inner();
    match AuthService::login(&request.email, &request.password, &db_context).await {
        Ok(token) => HttpResponse::Ok().body(token),
        Err(err) => HttpResponse::Unauthorized().body(err),
    }
}
