use serde::{Serialize, Deserialize};
use mongodb::bson::oid::ObjectId;

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct MarketPair {
    #[serde(rename = "_id", skip_serializing_if = "Option::is_none")]
    pub id: Option<ObjectId>,
    pub _exchange: ObjectId,
    pub _base_asset: ObjectId,
    pub _quote_asset: ObjectId,
    pub created_at: i64,
    pub updated_at: i64,
    pub status: bool,
}
