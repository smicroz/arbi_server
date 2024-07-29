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
    pub created_at: f64,
    pub updated_at: f64,
    pub status: bool,
}


impl MarketPairService {
    pub async fn create_market_pair(market_pair: MarketPair, db_context: &MongoDbContext) -> Result<MarketPair, String> {
        let db = db_context.get_database();
        let collection = db.collection::<MarketPair>("marketpairs");

        let now = Utc::now().timestamp() as f64;
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
        let collection = db.collection::<MarketPair>("marketpairs");

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
        let collection = db.collection::<MarketPair>("marketpairs");

        let now = Utc::now().timestamp() as f64;
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
        let collection = db.collection::<MarketPair>("marketpairs");

        collection.delete_one(doc! { "_id": id }).await
            .map_err(|e| {
                error!("Failed to delete market pair: {}", e);
                e.to_string()
            })?;

        Ok(())
    }

    pub async fn get_all_market_pairs_with_pagination(
        db_context: &MongoDbContext,
        page: u64,
        per_page: u64,
        exchange_id: Option<String>,
        search: Option<String>
    ) -> Result<(Vec<PopulatedMarketPair>, u64), String> {
        let db = db_context.get_database();
        let market_pairs_collection = db.collection::<Document>("marketpairs");
    
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
    pub async fn get_all_market_pairs_by_exchange(
        db_context: &MongoDbContext,
        exchange_id: ObjectId
    ) -> Result<Vec<PopulatedMarketPair>, String> {
        let db = db_context.get_database();
        let market_pairs_collection = db.collection::<Document>("marketpairs");
    
        let pipeline = vec![
            doc! { 
                "$match": { 
                    "_exchange": exchange_id 
                } 
            },
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
    
        let mut cursor = market_pairs_collection.aggregate(pipeline).await
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
    
        Ok(populated_market_pairs)
    }
    pub async fn get_conversion_pairs(
        db_context: &MongoDbContext,
        pair1: ObjectId,
        pair2: ObjectId
    ) -> Result<Vec<PopulatedMarketPair>, String> {
        let db = db_context.get_database();
        let market_pairs_collection = db.collection::<Document>("marketpairs");
    
        // Fetch the assets involved in pair1 and pair2
        let pair1_doc = market_pairs_collection.find_one(doc! { "_id": pair1 }).await
            .map_err(|e| e.to_string())?
            .ok_or_else(|| "Pair1 not found".to_string())?;
        let pair2_doc = market_pairs_collection.find_one(doc! { "_id": pair2 }).await
            .map_err(|e| e.to_string())?
            .ok_or_else(|| "Pair2 not found".to_string())?;
    
        let pair1: MarketPair = bson::from_document(pair1_doc).map_err(|e| e.to_string())?;
        let pair2: MarketPair = bson::from_document(pair2_doc).map_err(|e| e.to_string())?;
    
        let pipeline = vec![
            doc! {
                "$match": {
                    "$or": [
                        { "_base_asset": pair1._quote_asset, "_quote_asset": pair2._quote_asset },
                        { "_base_asset": pair2._quote_asset, "_quote_asset": pair1._quote_asset },
                    ]
                }
            },
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
    
        let mut cursor = market_pairs_collection.aggregate(pipeline).await
            .map_err(|e| e.to_string())?;
    
        let mut populated_market_pairs = Vec::new();
        while let Some(result) = cursor.try_next().await.map_err(|e| e.to_string())? {
            let populated_market_pair: PopulatedMarketPair = bson::from_document(result)
                .map_err(|e| e.to_string())?;
            populated_market_pairs.push(populated_market_pair);
        }
    
        Ok(populated_market_pairs)
    }
    // En src/modules/market_pair/market_pair_service.rs

    pub async fn get_conversion_pairs_for_arbitrage(
        db_context: &MongoDbContext,
        quote_asset1: &str,
        quote_asset2: &str
    ) -> Result<Vec<PopulatedMarketPair>, String> {
    let db = db_context.get_database();
    let market_pairs_collection = db.collection::<Document>("marketpairs");

    // Definir los posibles stablecoins y monedas fiat
    let stablecoins = doc! {
        "USD": ["USDT", "USDC", "DAI", "TUSD", "USDP", "GUSD", "USD"],
        "EUR": ["EURS", "EURT", "sEUR", "EURB", "EURe", "cEUR", "EUROC", "EUR"]
    };

    // Construir la lista de posibles assets para la conversión
    let mut possible_assets = vec![quote_asset1.to_string(), quote_asset2.to_string()];
    for (_, coins) in stablecoins {
        if let bson::Bson::Array(coin_array) = coins {
            for coin in coin_array.iter() {  // Usar .iter() aquí
                if coin.as_str().unwrap() == quote_asset1 || coin.as_str().unwrap() == quote_asset2 {
                    possible_assets.extend(coin_array.iter().map(|c| c.as_str().unwrap().to_string()));
                    break;
                }
            }
        }
    }

    // Eliminar duplicados
    possible_assets.sort();
    possible_assets.dedup();

    let pipeline = vec![
        doc! {
            "$match": {
                "$or": [
                    { "base_asset.short_name": { "$in": &possible_assets } },
                    { "quote_asset.short_name": { "$in": &possible_assets } }
                ]
            }
        },
        doc! {
            "$lookup": {
                "from": "exchanges",
                "localField": "_exchange",
                "foreignField": "_id",
                "as": "exchange"
            }
        },
        doc! { "$unwind": "$exchange" },
        doc! {
            "$project": {
                "_id": 1,
                "exchange": 1,
                "base_asset": 1,
                "quote_asset": 1,
                "created_at": 1,
                "updated_at": 1,
                "status": 1
            }
        }
    ];

    let mut cursor = market_pairs_collection.aggregate(pipeline).await
        .map_err(|e| e.to_string())?;

    let mut conversion_pairs = Vec::new();
    while let Some(result) = cursor.try_next().await.map_err(|e| e.to_string())? {
        let populated_market_pair: PopulatedMarketPair = bson::from_document(result)
            .map_err(|e| e.to_string())?;
        conversion_pairs.push(populated_market_pair);
    }

    Ok(conversion_pairs)
}
}
