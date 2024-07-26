use crate::db::mongodb::MongoDbContext;
use mongodb::bson::{doc, oid::ObjectId};
use crate::modules::exchange::exchange_schema::Exchange;
use tracing::error;
use chrono::Utc;
use futures::TryStreamExt;

pub struct ExchangeService;

impl ExchangeService {
    pub async fn create_exchange(exchange: Exchange, db_context: &MongoDbContext) -> Result<Exchange, String> {
        let db = db_context.get_database();
        let collection = db.collection::<Exchange>("exchanges");

        let now = Utc::now().timestamp() as f64;
        let new_exchange = Exchange {
            created_at: now,
            updated_at: now,
            ..exchange
        };

        let insert_result = collection.insert_one(new_exchange.clone()).await
            .map_err(|e| {
                error!("Failed to insert exchange: {}", e);
                e.to_string()
            })?;
        
        let new_exchange = collection.find_one(doc! { "_id": insert_result.inserted_id }).await
            .map_err(|e| {
                error!("Failed to fetch created exchange: {}", e);
                e.to_string()
            })?
            .ok_or_else(|| {
                let msg = "Failed to fetch created exchange".to_string();
                error!("{}", msg);
                msg
            })?;
        
        Ok(new_exchange)
    }

    pub async fn get_exchange(id: ObjectId, db_context: &MongoDbContext) -> Result<Exchange, String> {
        let db = db_context.get_database();
        let collection = db.collection::<Exchange>("exchanges");

        let exchange = collection.find_one(doc! { "_id": id }).await
            .map_err(|e| {
                error!("Failed to fetch exchange: {}", e);
                e.to_string()
            })?
            .ok_or_else(|| {
                let msg = "Exchange not found".to_string();
                error!("{}", msg);
                msg
            })?;

        Ok(exchange)
    }

    pub async fn update_exchange(id: ObjectId, updated_exchange: Exchange, db_context: &MongoDbContext) -> Result<Exchange, String> {
        let db = db_context.get_database();
        let collection = db.collection::<Exchange>("exchanges");

        let now = Utc::now().timestamp() as f64;
        let update_doc = doc! {
            "$set": {
                "name": updated_exchange.name,
                "short_name": updated_exchange.short_name,
                "url": updated_exchange.url,
                "updated_at": now,
            }
        };

        collection.update_one(doc! { "_id": id }, update_doc).await
            .map_err(|e| {
                error!("Failed to update exchange: {}", e);
                e.to_string()
            })?;

        Self::get_exchange(id, db_context).await
    }

    pub async fn delete_exchange(id: ObjectId, db_context: &MongoDbContext) -> Result<(), String> {
        let db = db_context.get_database();
        let collection = db.collection::<Exchange>("exchanges");

        collection.delete_one(doc! { "_id": id }).await
            .map_err(|e| {
                error!("Failed to delete exchange: {}", e);
                e.to_string()
            })?;

        Ok(())
    }

    pub async fn get_all_exchanges(db_context: &MongoDbContext) -> Result<Vec<Exchange>, String> {
        let db = db_context.get_database();
        let collection = db.collection::<Exchange>("exchanges");

        let mut cursor = collection.find(doc! {}).await
            .map_err(|e| {
                error!("Failed to fetch all exchanges: {}", e);
                e.to_string()
            })?;

        let mut exchanges = Vec::new();
        while let Some(exchange) = cursor.try_next().await.map_err(|e| {
            error!("Failed to iterate through exchanges: {}", e);
            e.to_string()
        })? {
            exchanges.push(exchange);
        }

        Ok(exchanges)
    }
}
