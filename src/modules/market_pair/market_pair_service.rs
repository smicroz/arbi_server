use crate::db::mongodb::MongoDbContext;
use mongodb::bson::{doc, Document, oid::ObjectId, Regex};
use crate::modules::market_pair::market_pair_schema::MarketPair;
use tracing::error;
use chrono::Utc;
use futures::TryStreamExt;
use serde::{Serialize, Deserialize};
use mongodb::bson;

use crate::modules::asset::asset_schema::Asset;
use crate::modules::exchange::exchange_schema::Exchange;

pub struct MarketPairService;

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct PopulatedMarketPair {
    #[serde(rename = "_id", skip_serializing_if = "Option::is_none")]
    pub id: Option<ObjectId>,
    pub exchange: Exchange,
    pub base_asset: Asset,
    pub quote_asset: Asset,
    pub created_at: i64,
    pub updated_at: i64,
    pub status: bool,
}


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

    pub async fn get_all_market_pairs(
        db_context: &MongoDbContext,
        page: u64,
        per_page: u64,
        exchange_id: Option<String>,
        search: Option<String>
    ) -> Result<(Vec<PopulatedMarketPair>, u64), String> {
        let db = db_context.get_database();
        let market_pairs_collection = db.collection::<Document>("market_pairs");
    
        let skip = (page - 1) * per_page;
    
        let mut initial_filter = doc! {};
    
        if let Some(exchange) = exchange_id.filter(|s| !s.is_empty()) {
            if let Ok(oid) = ObjectId::parse_str(&exchange) {
                initial_filter.insert("_exchange", oid);
            }
        }
    
        let mut pipeline = vec![
            doc! { "$match": initial_filter },
            doc! {
                "$lookup": {
                    "from": "exchanges",
                    "localField": "_exchange",
                    "foreignField": "_id",
                    "as": "exchange"
                }
            },
            doc! {
                "$lookup": {
                    "from": "assets",
                    "localField": "_base_asset",
                    "foreignField": "_id",
                    "as": "base_asset"
                }
            },
            doc! {
                "$lookup": {
                    "from": "assets",
                    "localField": "_quote_asset",
                    "foreignField": "_id",
                    "as": "quote_asset"
                }
            },
            doc! { "$unwind": "$exchange" },
            doc! { "$unwind": "$base_asset" },
            doc! { "$unwind": "$quote_asset" },
        ];
    
        if let Some(term) = search.filter(|s| !s.is_empty()) {
            let search_stage = doc! {
                "$match": {
                    "$or": [
                        { "base_asset.short_name": Regex { pattern: format!(".*{}.*", term), options: "i".to_string() } },
                        { "quote_asset.short_name": Regex { pattern: format!(".*{}.*", term), options: "i".to_string() } }
                    ]
                }
            };
            pipeline.push(search_stage);
        }
    
        pipeline.push(doc! { "$skip": skip as i64 });
        pipeline.push(doc! { "$limit": per_page as i64 });
    
        let mut cursor = market_pairs_collection.aggregate(pipeline.clone()).await
            .map_err(|e| {
                error!("Failed to aggregate market pairs: {}", e);
                e.to_string()
            })?;
    
        let mut populated_market_pairs = Vec::new();
        while let Some(result) = cursor.try_next().await.map_err(|e| {
            error!("Failed to iterate through aggregation results: {}", e);
            e.to_string()
        })? {
            let populated_market_pair: PopulatedMarketPair = bson::from_document(result)
                .map_err(|e| {
                    error!("Failed to deserialize market pair: {}", e);
                    e.to_string()
                })?;
    
            populated_market_pairs.push(populated_market_pair);
        }
    
        // Para contar el total, necesitamos quitar las etapas de paginación y agregar un $count
        pipeline.pop(); // Quitar $limit
        pipeline.pop(); // Quitar $skip
        pipeline.push(doc! { "$count": "total" });
    
        let total = market_pairs_collection.aggregate(pipeline).await
            .map_err(|e| {
                error!("Failed to count market pairs: {}", e);
                e.to_string()
            })?
            .try_next().await
            .map_err(|e| {
                error!("Failed to get count result: {}", e);
                e.to_string()
            })?
            .and_then(|doc| doc.get_i64("total").ok())
            .unwrap_or(0) as u64;
    
        Ok((populated_market_pairs, total))
    }
}
