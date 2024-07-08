mod db;
mod modules;
mod helpers;

use actix_web::{web, App, HttpServer};
use dotenv::dotenv;
use crate::db::mongodb::{get_mongodb_client, MongoDbContext};
use std::sync::Arc;
use tracing::{error, info};
use tracing_subscriber;

#[actix_web::main]
async fn main() -> std::io::Result<()> {
    dotenv().ok();
    tracing_subscriber::fmt::init(); // Inicializar el logger de tracing

    // Inicializar la conexión a MongoDB
    let client = match get_mongodb_client().await {
        Ok(client) => {
            info!("Successfully connected to MongoDB!");
            client
        },
        Err(e) => {
            error!("Failed to connect to MongoDB: {}", e);
            return Err(std::io::Error::new(std::io::ErrorKind::Other, format!("Failed to connect to MongoDB: {}", e)));
        }
    };

    // Crear el contexto de MongoDbContext
    let mongo_context = MongoDbContext::new(client);

    // Iniciar el servidor HTTP de Actix Web
    HttpServer::new(move || {
        App::new()
            .app_data(web::Data::new(mongo_context.clone()))
    })
    .bind("127.0.0.1:8081")?
    .run()
    .await
}
