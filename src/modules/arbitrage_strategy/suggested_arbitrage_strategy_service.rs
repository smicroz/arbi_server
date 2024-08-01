use crate::db::mongodb::MongoDbContext;
use mongodb::bson::{doc, oid::ObjectId, Document};
use crate::modules::arbitrage_strategy::arbitrage_strategy_schema::{ArbitrageStrategy, ArbitrageType, ArbitrageDetails, GeographicArbitrage};
use crate::modules::market_pair::market_pair_service::PopulatedMarketPair;
use futures::stream::TryStreamExt;
use mongodb::Collection;
use mongodb::bson;
use tracing::info;
pub struct SuggestedArbitrageStrategyService;

impl SuggestedArbitrageStrategyService {
    pub async fn get_suggested_strategies(
        db_context: &MongoDbContext,
        exchange1: ObjectId,
        exchange2: ObjectId,
        strategy_type: ArbitrageType,
    ) -> Result<Vec<ArbitrageStrategy>, String> {
        let db = db_context.get_database();
        let market_pairs_collection = db.collection::<Document>("marketpairs");
        let assets_collection = db.collection::<Document>("assets");

        match strategy_type {
            ArbitrageType::Geographic => {
                // Definir stablecoins y sus equivalencias
                let stablecoins = doc! {
                    "USD": ["USDT", "USDC", "BUSD", "DAI", "TUSD", "USDP", "GUSD", "FDUSD"],
                    "EUR": ["EURS", "EURT", "SEUR", "CEUR", "EURE", "JEUR"]
                };

                // Obtener pares de mercado para exchange1
                let exchange1_pairs = Self::get_exchange_pairs(&market_pairs_collection, exchange1).await?;
                info!("Found {} pairs in exchange1", exchange1_pairs.len());

                let mut suggested_strategies = Vec::new();

                for (index, pair1) in exchange1_pairs.iter().enumerate() {
                    info!("Processing pair {}/{} from exchange1: {}/{}", 
                          index + 1, exchange1_pairs.len(), 
                          pair1.base_asset.short_name, pair1.quote_asset.short_name);

                    // Buscar par correspondiente en exchange2
                    let pair2 = Self::find_corresponding_pair(&market_pairs_collection, &assets_collection, pair1, exchange2, &stablecoins).await?;

                    if let Some(pair2) = pair2 {
                        info!("Found corresponding pair in exchange2: {}/{}", 
                              pair2.base_asset.short_name, pair2.quote_asset.short_name);

                        // Buscar par de conversión
                        let conversion_pair = Self::find_conversion_pair(&market_pairs_collection, &assets_collection, pair1, &pair2, &stablecoins).await?;

                        if let Some(conversion_pair) = conversion_pair {
                            info!("Found conversion pair: {}/{}", 
                                  conversion_pair.base_asset.short_name, conversion_pair.quote_asset.short_name);

                            let strategy = ArbitrageStrategy {
                                id: None,
                                arbitrage_type: ArbitrageType::Geographic,
                                details: ArbitrageDetails::Geographic(GeographicArbitrage {
                                    pair1: pair1.id.unwrap(),
                                    pair2: pair2.id.unwrap(),
                                    conversion_pair: conversion_pair.id.unwrap(),
                                }),
                                created_at: 0.0,
                                updated_at: 0.0,
                                status: true,
                            };
                            suggested_strategies.push(strategy);
                            info!("Added new strategy to suggestions");
                        } else {
                            info!("No suitable conversion pair found");
                        }
                    } else {
                        info!("No corresponding pair found in exchange2");
                    }
                }

                info!("Total suggested strategies: {}", suggested_strategies.len());
                Ok(suggested_strategies)
            },
            _ => Err("Strategy type not implemented".to_string()),
        }
    }

    async fn find_corresponding_pair(
        market_pairs_collection: &Collection<Document>,
        assets_collection: &Collection<Document>,
        pair1: &PopulatedMarketPair,
        exchange2: ObjectId,
        stablecoins: &Document
    ) -> Result<Option<PopulatedMarketPair>, String> {
        info!("Searching for corresponding pair in exchange {} for {}/{}", 
              exchange2, pair1.base_asset.short_name, pair1.quote_asset.short_name);
    
        let get_asset_variants = |asset: &str| -> Vec<String> {
            let mut variants = vec![asset.to_string()];
            for (fiat, coins) in stablecoins.iter() {
                if let bson::Bson::Array(coin_array) = coins {
                    if coin_array.iter().any(|c| c.as_str().unwrap() == asset) {
                        variants.push(fiat.to_string());
                        variants.extend(coin_array.iter().map(|c| c.as_str().unwrap().to_string()));
                    }
                }
            }
            variants.sort();
            variants.dedup();
            variants
        };
    
        let quote_variants = get_asset_variants(&pair1.quote_asset.short_name);
    
        let pipeline = vec![
            doc! {
                "$match": {
                    "_exchange": exchange2
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
            doc! { "$unwind": "$base_asset" },
            doc! {
                "$lookup": {
                    "from": "assets",
                    "localField": "_quote_asset",
                    "foreignField": "_id",
                    "as": "quote_asset"
                }
            },
            doc! { "$unwind": "$quote_asset" },
            doc! {
                "$match": {
                    "base_asset.short_name": pair1.base_asset.short_name.clone(),
                    "quote_asset.short_name": { "$in": quote_variants }
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
            doc! { "$limit": 1 }
        ];
    
        let mut cursor = market_pairs_collection.aggregate(pipeline).await
            .map_err(|e| format!("Failed to execute aggregation: {}", e))?;
    
        if let Some(doc) = cursor.try_next().await
            .map_err(|e| format!("Error iterating cursor: {}", e))? {
            let corresponding_pair: PopulatedMarketPair = bson::from_document(doc)
                .map_err(|e| format!("Failed to deserialize market pair: {}", e))?;
            info!("Found corresponding pair: {}/{}", 
                  corresponding_pair.base_asset.short_name, corresponding_pair.quote_asset.short_name);
            Ok(Some(corresponding_pair))
        } else {
            info!("No corresponding pair found");
            Ok(None)
        }
    }

    async fn get_asset_ids(assets_collection: &Collection<Document>, asset_names: &[String]) -> Result<Vec<ObjectId>, String> {
        let pipeline = vec![
            doc! {
                "$match": {
                    "short_name": { "$in": asset_names }
                }
            },
            doc! {
                "$project": {
                    "_id": 1
                }
            }
        ];

        let mut cursor = assets_collection.aggregate(pipeline).await
            .map_err(|e| format!("Failed to execute aggregation: {}", e))?;

        let mut asset_ids = Vec::new();
        while let Some(doc) = cursor.try_next().await
            .map_err(|e| format!("Error iterating cursor: {}", e))? {
            if let Some(id) = doc.get_object_id("_id").ok() {
                asset_ids.push(id);
            }
        }

        Ok(asset_ids)
    }
    

    
        async fn find_conversion_pair(
            market_pairs_collection: &Collection<Document>,
            assets_collection: &Collection<Document>,
            pair1: &PopulatedMarketPair,
            pair2: &PopulatedMarketPair,
            stablecoins: &Document
        ) -> Result<Option<PopulatedMarketPair>, String> {
            info!("Searching for conversion pair between {}/{} and {}/{}", 
                  pair1.quote_asset.short_name, pair2.quote_asset.short_name,
                  pair1.base_asset.short_name, pair2.base_asset.short_name);
    
            let get_asset_variants = |asset: &str| -> Vec<String> {
                let mut variants = vec![asset.to_string()];
                for (fiat, coins) in stablecoins.iter() {
                    if let bson::Bson::Array(coin_array) = coins {
                        if coin_array.iter().any(|c| c.as_str().unwrap() == asset) {
                            variants.push(fiat.to_string());
                            variants.extend(coin_array.iter().map(|c| c.as_str().unwrap().to_string()));
                        }
                    }
                }
                variants.sort();
                variants.dedup();
                variants
            };
    
            let quote1_variants = get_asset_variants(&pair1.quote_asset.short_name);
            let quote2_variants = get_asset_variants(&pair2.quote_asset.short_name);
    
            let quote1_ids = Self::get_asset_ids(assets_collection, &quote1_variants).await?;
            let quote2_ids = Self::get_asset_ids(assets_collection, &quote2_variants).await?;
    
            let pipeline = vec![
                doc! {
                    "$match": {
                        "$or": [
                            {
                                "_base_asset": { "$in": &quote1_ids },
                                "_quote_asset": { "$in": &quote2_ids }
                            },
                            {
                                "_base_asset": { "$in": &quote2_ids },
                                "_quote_asset": { "$in": &quote1_ids }
                            }
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
                    "$lookup": {
                        "from": "assets",
                        "localField": "_base_asset",
                        "foreignField": "_id",
                        "as": "base_asset"
                    }
                },
                doc! { "$unwind": "$base_asset" },
                doc! {
                    "$lookup": {
                        "from": "assets",
                        "localField": "_quote_asset",
                        "foreignField": "_id",
                        "as": "quote_asset"
                    }
                },
                doc! { "$unwind": "$quote_asset" },
                doc! {
                    "$addFields": {
                        "priority": {
                            "$switch": {
                                "branches": [
                                    { "case": { "$in": ["$base_asset.short_name", ["USD", "EUR"]] }, "then": 1 },
                                    { "case": { "$in": ["$quote_asset.short_name", ["USD", "EUR"]] }, "then": 1 },
                                    { "case": { "$in": ["$base_asset.short_name", ["USDT", "USDC", "BUSD", "DAI", "TUSD", "USDP", "GUSD", "FDUSD", "EURS", "EURT"]] }, "then": 2 },
                                    { "case": { "$in": ["$quote_asset.short_name", ["USDT", "USDC", "BUSD", "DAI", "TUSD", "USDP", "GUSD", "FDUSD", "EURS", "EURT"]] }, "then": 2 }
                                ],
                                "default": 3
                            }
                        }
                    }
                },
                doc! { "$sort": { "priority": 1 } },
                doc! { "$limit": 1 }
            ];
    
            let mut cursor = market_pairs_collection.aggregate(pipeline).await
                .map_err(|e| format!("Failed to execute aggregation: {}", e))?;
    
            if let Some(doc) = cursor.try_next().await
                .map_err(|e| format!("Error iterating cursor: {}", e))? {
                let conversion_pair: PopulatedMarketPair = bson::from_document(doc)
                    .map_err(|e| format!("Failed to deserialize market pair: {}", e))?;
                info!("Found conversion pair: {}/{}", 
                      conversion_pair.base_asset.short_name, conversion_pair.quote_asset.short_name);
                Ok(Some(conversion_pair))
            } else {
                info!("No conversion pair found");
                Ok(None)
            }
        }
        async fn get_exchange_pairs(
            collection: &Collection<Document>,
            exchange_id: ObjectId
        ) -> Result<Vec<PopulatedMarketPair>, String> {
            let pipeline = vec![
                // Match para filtrar por el exchange específico
                doc! {
                    "$match": {
                        "_exchange": exchange_id
                    }
                },
                // Lookup para obtener la información del exchange
                doc! {
                    "$lookup": {
                        "from": "exchanges",
                        "localField": "_exchange",
                        "foreignField": "_id",
                        "as": "exchange"
                    }
                },
                // Unwind del array de exchange (debería ser solo uno)
                doc! {
                    "$unwind": "$exchange"
                },
                // Lookup para obtener la información del activo base
                doc! {
                    "$lookup": {
                        "from": "assets",
                        "localField": "_base_asset",
                        "foreignField": "_id",
                        "as": "base_asset"
                    }
                },
                // Unwind del array de base_asset
                doc! {
                    "$unwind": "$base_asset"
                },
                // Lookup para obtener la información del activo quote
                doc! {
                    "$lookup": {
                        "from": "assets",
                        "localField": "_quote_asset",
                        "foreignField": "_id",
                        "as": "quote_asset"
                    }
                },
                // Unwind del array de quote_asset
                doc! {
                    "$unwind": "$quote_asset"
                },
                // Proyección para dar formato a la salida
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
    
            let mut cursor = collection.aggregate(pipeline).await
                .map_err(|e| format!("Failed to execute aggregation: {}", e))?;
    
            let mut market_pairs = Vec::new();
    
            while let Some(doc) = cursor.try_next().await
                .map_err(|e| format!("Error iterating cursor: {}", e))? {
                let market_pair: PopulatedMarketPair = bson::from_document(doc)
                    .map_err(|e| format!("Failed to deserialize market pair: {}", e))?;
                market_pairs.push(market_pair);
            }
    
            Ok(market_pairs)
        }
    
}