use mongodb::{Client as MongoClient, options::ClientOptions, Database};
use std::env;

#[derive(Clone)]
pub struct MongoDbContext {
    pub client: MongoClient,
    pub db_name: String,
}

impl MongoDbContext {
    pub fn new(client: MongoClient) -> Self {
        let db_name = env::var("MONGODB_DBNAME").expect("MONGODB_DBNAME must be set");
        Self { client, db_name }
    }

    pub fn get_database(&self) -> Database {
        self.client.database(&self.db_name)
    }
}

pub async fn get_mongodb_client() -> Result<MongoClient, mongodb::error::Error> {
    let username = env::var("MONGODB_USERNAME").expect("MONGODB_USERNAME must be set");
    let password = env::var("MONGODB_PASSWORD").expect("MONGODB_PASSWORD must be set");
    let hostname = env::var("MONGODB_HOSTNAME").expect("MONGODB_HOSTNAME must be set");
    let port = env::var("MONGODB_PORT").expect("MONGODB_PORT must be set");
    let dbname = env::var("MONGODB_DBNAME").expect("MONGODB_DBNAME must be set");

    let mongo_uri = format!(
        "mongodb://{}:{}@{}:{}/{}?authSource=admin",
        username, password, hostname, port, dbname
    );

    let mut client_options = ClientOptions::parse(&mongo_uri).await?;
    client_options.app_name = Some(dbname.clone());

    let client = MongoClient::with_options(client_options)?;
    Ok(client)
}
