use serde::{Serialize, Deserialize};
use mongodb::bson::oid::ObjectId;

#[derive(Clone, Serialize, Deserialize, Debug)]
pub struct Exchange {
    #[serde(rename = "_id", skip_serializing_if = "Option::is_none")]
    pub id: Option<ObjectId>,
    pub name: String,
    pub short_name: String,
    pub url: String,
    pub created_at: i64,
    pub updated_at: i64,
}
