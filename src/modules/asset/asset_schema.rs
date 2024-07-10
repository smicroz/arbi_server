use serde::{Serialize, Deserialize};
use mongodb::bson::oid::ObjectId;

#[derive(Serialize, Deserialize, Debug, Clone)] // Añadir Clone
pub struct Asset {
    #[serde(rename = "_id", skip_serializing_if = "Option::is_none")]
    pub id: Option<ObjectId>,
    pub _exchange_id: ObjectId,
    pub name: String,
    pub short_name: String,
    pub created_at: i64,
    pub updated_at: i64,
    pub status: bool,
}
