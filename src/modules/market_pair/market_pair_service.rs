use crate::db::mongodb::MongoDbContext;
use mongodb::bson::{doc, oid::ObjectId};
use crate::modules::market_pair::market_pair_schema::MarketPair;
use tracing::error;
use chrono::Utc;
use futures::TryStreamExt;

pub struct MarketPairService;

impl MarketPairService {
    pub async fn create_market_pair(market_pair: MarketPair, db_context: &MongoDbContext) -> Result<MarketPair, String> {
        let db = db_context.get_database();
        let collection = db.collection::<MarketPair>("market_pairs");

        let now = Utc::now().timestamp();
        let new_market_pair = MarketPair {
            created_at: now,
            updated_at: now,
            ..market_pair
        };

        let insert_result = collection.insert_one(new_market_pair.clone()).await
            .map_err(|e| {
                error!("Failed to insert market pair: {}", e);
                e.to_string()
            })?;
        
        let new_market_pair = collection.find_one(doc! { "_id": insert_result.inserted_id }).await
            .map_err(|e| {
                error!("Failed to fetch created market pair: {}", e);
                e.to_string()
            })?
            .ok_or_else(|| {
                let msg = "Failed to fetch created market pair".to_string();
                error!("{}", msg);
                msg
            })?;
        
        Ok(new_market_pair)
    }

    pub async fn get_market_pair(id: ObjectId, db_context: &MongoDbContext) -> Result<MarketPair, String> {
        let db = db_context.get_database();
        let collection = db.collection::<MarketPair>("market_pairs");

        let market_pair = collection.find_one(doc! { "_id": id }).await
            .map_err(|e| {
                error!("Failed to fetch market pair: {}", e);
                e.to_string()
            })?
            .ok_or_else(|| {
                let msg = "Market pair not found".to_string();
                error!("{}", msg);
                msg
            })?;

        Ok(market_pair)
    }

    pub async fn update_market_pair(id: ObjectId, updated_market_pair: MarketPair, db_context: &MongoDbContext) -> Result<MarketPair, String> {
        let db = db_context.get_database();
        let collection = db.collection::<MarketPair>("market_pairs");

        let now = Utc::now().timestamp();
        let update_doc = doc! {
            "$set": {
                "_exchange": updated_market_pair._exchange,
                "_base_asset": updated_market_pair._base_asset,
                "_quote_asset": updated_market_pair._quote_asset,
                "updated_at": now,
                "status": updated_market_pair.status,
            }
        };

        collection.update_one(doc! { "_id": id }, update_doc).await
            .map_err(|e| {
                error!("Failed to update market pair: {}", e);
                e.to_string()
            })?;

        Self::get_market_pair(id, db_context).await
    }

    pub async fn delete_market_pair(id: ObjectId, db_context: &MongoDbContext) -> Result<(), String> {
        let db = db_context.get_database();
        let collection = db.collection::<MarketPair>("market_pairs");

        collection.delete_one(doc! { "_id": id }).await
            .map_err(|e| {
                error!("Failed to delete market pair: {}", e);
                e.to_string()
            })?;

        Ok(())
    }

    pub async fn get_all_market_pairs(db_context: &MongoDbContext) -> Result<Vec<MarketPair>, String> {
        let db = db_context.get_database();
        let collection = db.collection::<MarketPair>("market_pairs");

        let mut cursor = collection.find(doc! {}).await
            .map_err(|e| {
                error!("Failed to fetch all market pairs: {}", e);
                e.to_string()
            })?;

        let mut market_pairs = Vec::new();
        while let Some(market_pair) = cursor.try_next().await.map_err(|e| {
            error!("Failed to iterate through market pairs: {}", e);
            e.to_string()
        })? {
            market_pairs.push(market_pair);
        }

        Ok(market_pairs)
    }
}
