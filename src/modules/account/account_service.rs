use crate::modules::user::user_schema::User;
use mongodb::bson::{doc, oid::ObjectId};
use crate::db::mongodb::MongoDbContext;
use tracing::{ error};
use serde::{Serialize, Deserialize};

#[derive(Serialize, Deserialize)]
pub struct UpdateUserRequest {
    pub name: Option<String>,
    pub email: Option<String>,
    pub password: Option<String>,
    pub _default_asset: Option<String>, // Mantén este campo como String para recibirlo desde el frontend
    pub _default_market_pair: Option<String>, // Mantén este campo como String para recibirlo desde el frontend
}

pub struct AccountService;

impl AccountService {
    pub async fn get_user(user_id: ObjectId, db_context: &MongoDbContext) -> Result<User, String> {
        let db = db_context.get_database();
        let collection = db.collection::<User>("users");

        let user = collection
            .find_one(doc! { "_id": user_id })
            .await
            .map_err(|e| e.to_string())?
            .ok_or_else(|| {
                error!("User not found: {}", user_id);
                "User not found".to_string()
            })?;

        Ok(user)
    }

    pub async fn update_user(user_id: ObjectId, update_data: UpdateUserRequest, db_context: &MongoDbContext) -> Result<User, String> {
        let db = db_context.get_database();
        let collection = db.collection::<User>("users");

        let mut update_doc = doc! {};


        if let Some(name) = update_data.name {
            update_doc.insert("name", name);
        }

        if let Some(email) = update_data.email {
            update_doc.insert("email", email);
        }

        // Solo actualiza la contraseña si se proporciona y no está vacía
        if let Some(password) = update_data.password {
            if !password.is_empty() {
                let hashed_password = bcrypt::hash(password, bcrypt::DEFAULT_COST).map_err(|e| e.to_string())?;
                update_doc.insert("password", hashed_password);
            }
        }

        println!("{:?}", update_data._default_asset);
        if let Some(default_asset) = update_data._default_asset {
            // Convertir el default_asset de String a ObjectId
            let default_asset_id = ObjectId::parse_str(&default_asset).map_err(|e| e.to_string())?;
            update_doc.insert("_default_asset", default_asset_id);
        }

        // Solo realiza la actualización si hay campos para actualizar
        if !update_doc.is_empty() {
            collection
                .update_one(doc! { "_id": user_id }, doc! { "$set": update_doc })
                .await
                .map_err(|e| e.to_string())?;
        }

        Self::get_user(user_id, db_context).await
    }
}
