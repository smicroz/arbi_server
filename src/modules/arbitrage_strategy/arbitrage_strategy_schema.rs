use serde::{Serialize, Deserialize};
use mongodb::bson::oid::ObjectId;

#[derive(Serialize, Deserialize, Debug, Clone)]
pub enum ArbitrageType {
    Geographic,
    Exchange,
    Triangular,
    TradingPair,
}

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct ArbitrageStrategy {
    #[serde(rename = "_id", skip_serializing_if = "Option::is_none")]
    pub id: Option<ObjectId>,
    pub arbitrage_type: ArbitrageType,
    pub details: ArbitrageDetails,
    pub created_at: f64,
    pub updated_at: f64,
    pub status: bool,
}

#[derive(Serialize, Deserialize, Debug, Clone)]
pub enum ArbitrageDetails {
    Geographic(GeographicArbitrage),
    Exchange(ExchangeArbitrage),
    Triangular(TriangularArbitrage),
    TradingPair(TradingPairArbitrage),
}

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct GeographicArbitrage {
    pub pair1: ObjectId,
    pub pair2: ObjectId,
    pub conversion_pair: ObjectId,
}

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct ExchangeArbitrage {
    pub pair1: ObjectId,
    pub pair2: ObjectId,
}

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct TriangularArbitrage {
    pub pair1: ObjectId,
    pub pair2: ObjectId,
    pub pair3: ObjectId,
}

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct TradingPairArbitrage {
    pub pair1: ObjectId,
    pub pair2: ObjectId,
    pub pair3: ObjectId,
}