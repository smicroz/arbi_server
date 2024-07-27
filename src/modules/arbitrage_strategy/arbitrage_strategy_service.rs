use crate::db::mongodb::MongoDbContext;
use mongodb::bson::{doc, oid::ObjectId};
use crate::modules::arbitrage_strategy::arbitrage_strategy_schema::{ArbitrageStrategy, ArbitrageType};
use tracing::error;
use chrono::Utc;
use futures::TryStreamExt;
use mongodb::bson;

pub struct ArbitrageStrategyService;

impl ArbitrageStrategyService {
    pub async fn create_arbitrage_strategy(strategy: ArbitrageStrategy, db_context: &MongoDbContext) -> Result<ArbitrageStrategy, String> {
        let db = db_context.get_database();
        let collection = db.collection::<ArbitrageStrategy>("arbitrage_strategies");

        let now = Utc::now().timestamp() as f64;
        let new_strategy = ArbitrageStrategy {
            created_at: now,
            updated_at: now,
            ..strategy
        };

        let insert_result = collection.insert_one(new_strategy.clone()).await
            .map_err(|e| {
                error!("Failed to insert arbitrage strategy: {}", e);
                e.to_string()
            })?;
        
        let new_strategy = collection.find_one(doc! { "_id": insert_result.inserted_id }).await
            .map_err(|e| {
                error!("Failed to fetch created arbitrage strategy: {}", e);
                e.to_string()
            })?
            .ok_or_else(|| {
                let msg = "Failed to fetch created arbitrage strategy".to_string();
                error!("{}", msg);
                msg
            })?;
        
        Ok(new_strategy)
    }

    pub async fn get_arbitrage_strategy(id: ObjectId, db_context: &MongoDbContext) -> Result<ArbitrageStrategy, String> {
        let db = db_context.get_database();
        let collection = db.collection::<ArbitrageStrategy>("arbitrage_strategies");

        let strategy = collection.find_one(doc! { "_id": id }).await
            .map_err(|e| {
                error!("Failed to fetch arbitrage strategy: {}", e);
                e.to_string()
            })?
            .ok_or_else(|| {
                let msg = "Arbitrage strategy not found".to_string();
                error!("{}", msg);
                msg
            })?;

        Ok(strategy)
    }

    pub async fn update_arbitrage_strategy(id: ObjectId, updated_strategy: ArbitrageStrategy, db_context: &MongoDbContext) -> Result<ArbitrageStrategy, String> {
        let db = db_context.get_database();
        let collection = db.collection::<ArbitrageStrategy>("arbitrage_strategies");

        let now = Utc::now().timestamp() as f64;
        let update_doc = doc! {
            "$set": {
                "arbitrage_type": bson::to_bson(&updated_strategy.arbitrage_type)
                    .map_err(|e| e.to_string())?,
                "details": bson::to_bson(&updated_strategy.details)
                    .map_err(|e| e.to_string())?,
                "updated_at": now,
                "status": updated_strategy.status,
            }
        };

        collection.update_one(doc! { "_id": id }, update_doc).await
            .map_err(|e| {
                error!("Failed to update arbitrage strategy: {}", e);
                e.to_string()
            })?;

        Self::get_arbitrage_strategy(id, db_context).await
    }

    pub async fn delete_arbitrage_strategy(id: ObjectId, db_context: &MongoDbContext) -> Result<(), String> {
        let db = db_context.get_database();
        let collection = db.collection::<ArbitrageStrategy>("arbitrage_strategies");

        collection.delete_one(doc! { "_id": id }).await
            .map_err(|e| {
                error!("Failed to delete arbitrage strategy: {}", e);
                e.to_string()
            })?;

        Ok(())
    }

    pub async fn get_all_arbitrage_strategies(
        db_context: &MongoDbContext,
        page: u64,
        per_page: u64,
        arbitrage_type: Option<ArbitrageType>,
    ) -> Result<(Vec<ArbitrageStrategy>, u64), String> {
        let db = db_context.get_database();
        let collection = db.collection::<ArbitrageStrategy>("arbitrage_strategies");

        let skip = (page - 1) * per_page;
        
        let mut filter = doc! {};
        if let Some(arb_type) = arbitrage_type {
            filter.insert("arbitrage_type", bson::to_bson(&arb_type)
                .map_err(|e| e.to_string())?);
        }

        let mut cursor = collection.find(filter.clone()).skip(skip as u64).limit(per_page as i64).await
            .map_err(|e| {
                error!("Failed to fetch arbitrage strategies: {}", e);
                e.to_string()
            })?;

        let mut strategies = Vec::new();
        while let Some(strategy) = cursor.try_next().await.map_err(|e| {
            error!("Failed to iterate through arbitrage strategies: {}", e);
            e.to_string()
        })? {
            strategies.push(strategy);
        }

        let total = collection.count_documents(filter).await
            .map_err(|e| {
                error!("Failed to count arbitrage strategies: {}", e);
                e.to_string()
            })?;

        Ok((strategies, total))
    }
}