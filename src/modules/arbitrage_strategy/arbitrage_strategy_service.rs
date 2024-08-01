use crate::db::mongodb::MongoDbContext;
use mongodb::bson::{doc, oid::ObjectId, Document};
use crate::modules::arbitrage_strategy::arbitrage_strategy_schema::{ArbitrageStrategy, ArbitrageType, ArbitrageDetails};
use crate::modules::market_pair::market_pair_service::PopulatedMarketPair;
use chrono::Utc;
use futures::TryStreamExt;
use mongodb::bson;
use tracing::{error, info};

#[derive(serde::Serialize, serde::Deserialize, Debug, Clone)]
pub struct PopulatedArbitrageStrategy {
    #[serde(rename = "_id")]
    pub id: ObjectId,
    pub arbitrage_type: ArbitrageType,
    pub details: PopulatedArbitrageDetails,
    pub created_at: f64,
    pub updated_at: f64,
    pub status: bool,
}

#[derive(serde::Serialize, serde::Deserialize, Debug, Clone)]
pub enum PopulatedArbitrageDetails {
    Geographic {
        pair1: PopulatedMarketPair,
        pair2: PopulatedMarketPair,
        conversion_pair: PopulatedMarketPair,
    },
    Exchange {
        pair1: PopulatedMarketPair,
        pair2: PopulatedMarketPair,
    },
    Triangular {
        pair1: PopulatedMarketPair,
        pair2: PopulatedMarketPair,
        pair3: PopulatedMarketPair,
    },
    TradingPair {
        pair1: PopulatedMarketPair,
        pair2: PopulatedMarketPair,
        pair3: PopulatedMarketPair,
    },
}

pub struct ArbitrageStrategyService;

impl ArbitrageStrategyService {


    

    pub async fn create_arbitrage_strategy(mut strategy: ArbitrageStrategy, db_context: &MongoDbContext) -> Result<ArbitrageStrategy, String> {
        let db = db_context.get_database();
        let collection = db.collection::<ArbitrageStrategy>("arbitrage_strategies");
    
        let now = Utc::now().timestamp() as f64;
        strategy.created_at = now;
        strategy.updated_at = now;
    
        // Log the received strategy
        info!("Received strategy: {:?}", strategy);
        
        // Función auxiliar para verificar si un ObjectId es válido
        fn is_valid_object_id(id: &ObjectId) -> bool {
            // Un ObjectId siempre debe tener 12 bytes
            id.bytes().len() == 12 && !id.to_hex().chars().all(|c| c == '0')
        }

        // Procesar los detalles según el tipo de arbitraje
        match &mut strategy.details {
            ArbitrageDetails::Geographic(geo) => {
                if !is_valid_object_id(&geo.pair1) || !is_valid_object_id(&geo.pair2) || !is_valid_object_id(&geo.conversion_pair) {
                    return Err("Invalid ObjectId for Geographic arbitrage".to_string());
                }
                // Aquí podrías agregar más validaciones específicas para el arbitraje geográfico
            },
            ArbitrageDetails::Exchange(ex) => {
                if !is_valid_object_id(&ex.pair1) || !is_valid_object_id(&ex.pair2) {
                    return Err("Invalid ObjectId for Exchange arbitrage".to_string());
                }
                // Validaciones adicionales para el arbitraje de intercambio
            },
            ArbitrageDetails::Triangular(tri) => {
                if !is_valid_object_id(&tri.pair1) || !is_valid_object_id(&tri.pair2) || !is_valid_object_id(&tri.pair3) {
                    return Err("Invalid ObjectId for Triangular arbitrage".to_string());
                }
                // Validaciones específicas para el arbitraje triangular
                if tri.pair1 == tri.pair2 || tri.pair2 == tri.pair3 || tri.pair1 == tri.pair3 {
                    return Err("All pairs must be different for Triangular arbitrage".to_string());
                }
            },
            ArbitrageDetails::TradingPair(tp) => {
                if !is_valid_object_id(&tp.pair1) || !is_valid_object_id(&tp.pair2) || !is_valid_object_id(&tp.pair3) {
                    return Err("Invalid ObjectId for Trading Pair arbitrage".to_string());
                }
                // Validaciones adicionales para el arbitraje de pares de trading
            },
        }
    
        let insert_result = collection.insert_one(strategy.clone()).await
            .map_err(|e| {
                error!("Failed to insert arbitrage strategy: {}", e);
                e.to_string()
            })?;
        
        info!("Insert result: {:?}", insert_result);
    
        let created_strategy = collection.find_one(doc! { "_id": insert_result.inserted_id }).await
            .map_err(|e| {
                error!("Failed to fetch created arbitrage strategy: {}", e);
                e.to_string()
            })?
            .ok_or_else(|| {
                let msg = "Failed to fetch created arbitrage strategy".to_string();
                error!("{}", msg);
                msg
            })?;
        
        info!("Created strategy: {:?}", created_strategy);
    
        Ok(created_strategy)
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
    ) -> Result<(Vec<PopulatedArbitrageStrategy>, u64), String> {
        let db = db_context.get_database();
        let collection = db.collection::<Document>("arbitrage_strategies");
    
        let skip = (page - 1) * per_page;
        
        let mut filter = doc! {};
        if let Some(arb_type) = arbitrage_type {
            filter.insert("arbitrage_type", bson::to_bson(&arb_type).map_err(|e| e.to_string())?);
        }
    
        let pipeline = vec![
            doc! { "$match": filter.clone() },
            doc! { "$skip": skip as i64 },
            doc! { "$limit": per_page as i64 },
            doc! {
                "$lookup": {
                    "from": "marketpairs",
                    "let": { 
                        "pair1": "$details.Geographic.pair1", 
                        "pair2": "$details.Geographic.pair2", 
                        "conversion_pair": "$details.Geographic.conversion_pair",
                        "exchange_pair1": "$details.Exchange.pair1",
                        "exchange_pair2": "$details.Exchange.pair2",
                        "triangular_pair1": "$details.Triangular.pair1",
                        "triangular_pair2": "$details.Triangular.pair2",
                        "triangular_pair3": "$details.Triangular.pair3",
                        "trading_pair1": "$details.TradingPair.pair1",
                        "trading_pair2": "$details.TradingPair.pair2",
                        "trading_pair3": "$details.TradingPair.pair3"
                    },
                    "pipeline": [
                        { "$match": 
                            { "$expr": 
                                { "$or": [
                                    { "$eq": ["$_id", "$$pair1"] },
                                    { "$eq": ["$_id", "$$pair2"] },
                                    { "$eq": ["$_id", "$$conversion_pair"] },
                                    { "$eq": ["$_id", "$$exchange_pair1"] },
                                    { "$eq": ["$_id", "$$exchange_pair2"] },
                                    { "$eq": ["$_id", "$$triangular_pair1"] },
                                    { "$eq": ["$_id", "$$triangular_pair2"] },
                                    { "$eq": ["$_id", "$$triangular_pair3"] },
                                    { "$eq": ["$_id", "$$trading_pair1"] },
                                    { "$eq": ["$_id", "$$trading_pair2"] },
                                    { "$eq": ["$_id", "$$trading_pair3"] }
                                ]}
                            }
                        },
                        {
                            "$lookup": {
                                "from": "exchanges",
                                "localField": "_exchange",
                                "foreignField": "_id",
                                "as": "exchange"
                            }
                        },
                        { "$unwind": "$exchange" },
                        {
                            "$lookup": {
                                "from": "assets",
                                "localField": "_base_asset",
                                "foreignField": "_id",
                                "as": "base_asset"
                            }
                        },
                        { "$unwind": "$base_asset" },
                        {
                            "$lookup": {
                                "from": "assets",
                                "localField": "_quote_asset",
                                "foreignField": "_id",
                                "as": "quote_asset"
                            }
                        },
                        { "$unwind": "$quote_asset" }
                    ],
                    "as": "populated_pairs"
                }
            }
        ];
    
        let mut cursor = collection.aggregate(pipeline).await
            .map_err(|e| {
                error!("Failed to fetch arbitrage strategies: {}", e);
                e.to_string()
            })?;
    
        let mut strategies = Vec::new();
        while let Some(doc) = cursor.try_next().await.map_err(|e| {
            error!("Failed to iterate through arbitrage strategies: {}", e);
            e.to_string()
        })? {
            let strategy: ArbitrageStrategy = bson::from_document(doc.clone())
                .map_err(|e| {
                    error!("Failed to deserialize arbitrage strategy: {}", e);
                    e.to_string()
                })?;
    
            let populated_pairs: Vec<PopulatedMarketPair> = bson::from_bson(bson::Bson::Array(doc.get_array("populated_pairs").unwrap_or(&Vec::new()).clone()))
                .map_err(|e| {
                    error!("Failed to deserialize populated pairs: {}", e);
                    e.to_string()
                })?;
    
            let populated_details = match &strategy.details {
                ArbitrageDetails::Geographic(geo) => {
                    let pair1 = populated_pairs.iter().find(|p| p.id == Some(geo.pair1)).cloned();
                    let pair2 = populated_pairs.iter().find(|p| p.id == Some(geo.pair2)).cloned();
                    let conversion_pair = populated_pairs.iter().find(|p| p.id == Some(geo.conversion_pair)).cloned();
                    
                    if let (Some(p1), Some(p2), Some(cp)) = (pair1, pair2, conversion_pair) {
                        PopulatedArbitrageDetails::Geographic { pair1: p1, pair2: p2, conversion_pair: cp }
                    } else {
                        continue;
                    }
                },
                ArbitrageDetails::Exchange(ex) => {
                    let pair1 = populated_pairs.iter().find(|p| p.id == Some(ex.pair1)).cloned();
                    let pair2 = populated_pairs.iter().find(|p| p.id == Some(ex.pair2)).cloned();
                    
                    if let (Some(p1), Some(p2)) = (pair1, pair2) {
                        PopulatedArbitrageDetails::Exchange { pair1: p1, pair2: p2 }
                    } else {
                        continue;
                    }
                },
                ArbitrageDetails::Triangular(tri) => {
                    let pair1 = populated_pairs.iter().find(|p| p.id == Some(tri.pair1)).cloned();
                    let pair2 = populated_pairs.iter().find(|p| p.id == Some(tri.pair2)).cloned();
                    let pair3 = populated_pairs.iter().find(|p| p.id == Some(tri.pair3)).cloned();
                    
                    if let (Some(p1), Some(p2), Some(p3)) = (pair1, pair2, pair3) {
                        PopulatedArbitrageDetails::Triangular { pair1: p1, pair2: p2, pair3: p3 }
                    } else {
                        continue;
                    }
                },
                ArbitrageDetails::TradingPair(tp) => {
                    let pair1 = populated_pairs.iter().find(|p| p.id == Some(tp.pair1)).cloned();
                    let pair2 = populated_pairs.iter().find(|p| p.id == Some(tp.pair2)).cloned();
                    let pair3 = populated_pairs.iter().find(|p| p.id == Some(tp.pair3)).cloned();
                    
                    if let (Some(p1), Some(p2), Some(p3)) = (pair1, pair2, pair3) {
                        PopulatedArbitrageDetails::TradingPair { pair1: p1, pair2: p2, pair3: p3 }
                    } else {
                        continue;
                    }
                },
            };
    
            let populated_strategy = PopulatedArbitrageStrategy {
                id: strategy.id.unwrap(),
                arbitrage_type: strategy.arbitrage_type,
                details: populated_details,
                created_at: strategy.created_at,
                updated_at: strategy.updated_at,
                status: strategy.status,
            };
    
            strategies.push(populated_strategy);
        }
    
        let total = collection.count_documents(filter).await
            .map_err(|e| {
                error!("Failed to count arbitrage strategies: {}", e);
                e.to_string()
            })?;
    
        Ok((strategies, total))
    }
}