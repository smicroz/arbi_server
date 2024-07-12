use serde::{Serialize, Deserialize};
use mongodb::bson::oid::ObjectId;
use chrono::NaiveDateTime;

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct User {
    #[serde(rename = "_id", skip_serializing_if = "Option::is_none")]
    pub id: Option<ObjectId>,
    pub name: String,
    pub email: String,
    pub password: String,
    pub _default_asset: Option<ObjectId>,
    pub _default_market_pair: Option<ObjectId>,
    pub password_reset_token: String,
    pub password_reset_expires: NaiveDateTime,
    pub tokens: Vec<String>, // Almacenar tokens activos
    pub role: String, // Almacenar el rol del usuario (e.g., "user", "admin")
}
