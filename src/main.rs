mod db;
mod modules;
mod helpers;
mod middleware;
mod router; // Importa el archivo router.rs

use actix_web::{web, App, HttpServer};
use dotenv::dotenv;
use crate::db::mongodb::{get_mongodb_client, MongoDbContext};
use tracing::{error, info};
use tracing_subscriber;
use crate::middleware::auth_middleware::Auth;

#[actix_web::main]
async fn main() -> std::io::Result<()> {
    dotenv().ok();

    // Configurar tracing con variables de entorno
    tracing_subscriber::fmt()
        .with_env_filter(tracing_subscriber::EnvFilter::from_default_env())
        .init();

    // Inicializar la conexión a MongoDB
    let client = match get_mongodb_client().await {
        Ok(client) => {
            info!("Successfully connected to MongoDB!");
            client
        },
        Err(e) => {
            error!("Failed to connect to MongoDb: {}", e);
            return Err(std::io::Error::new(std::io::ErrorKind::Other, format!("Failed to connect to MongoDB: {}", e)));
        }
    };

    // Crear el contexto de MongoDbContext
    let mongo_context = MongoDbContext::new(client);

    // Iniciar el servidor HTTP de Actix Web
    HttpServer::new(move || {
        App::new()
            .wrap(Auth) // Añadir el middleware de autenticación
            .app_data(web::Data::new(mongo_context.clone())) // Pasar el contexto de MongoDbContext al contexto de Actix Web
            .configure(router::configure) // Configurar las rutas usando router.rs
    })
    .bind("127.0.0.1:8081")?
    .run()
    .await
}
