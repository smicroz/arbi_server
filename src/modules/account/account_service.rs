use crate::modules::user::user_schema::User;
use mongodb::bson::{doc, oid::ObjectId};
use crate::db::mongodb::MongoDbContext;
use tracing::{info, error}; // Mantenemos las importaciones de info y error
use serde::{Serialize, Deserialize};

#[derive(Serialize, Deserialize)]
pub struct UpdateUserRequest {
    pub email: Option<String>,
    pub password: Option<String>,
    pub default_asset: Option<ObjectId>,
}

pub struct AccountService;

impl AccountService {
    pub async fn get_user(user_id: ObjectId, db_context: &MongoDbContext) -> Result<User, String> {
        let db = db_context.get_database();
        let collection = db.collection::<User>("users");

        let user = collection
            .find_one(doc! { "_id": user_id })
            .await
            .map_err(|e| {
                error!("Failed to fetch user: {}", e);
                e.to_string()
            })?
            .ok_or_else(|| {
                let msg = format!("User not found: {}", user_id);
                error!("{}", msg);
                msg
            })?;

        info!("Fetched user: {}", user_id); // Registrar información cuando se obtiene el usuario con éxito
        Ok(user)
    }

    pub async fn update_user(user_id: ObjectId, update_data: UpdateUserRequest, db_context: &MongoDbContext) -> Result<User, String> {
        let db = db_context.get_database();
        let collection = db.collection::<User>("users");

        let mut update_doc = doc! {};
        if let Some(email) = update_data.email {
            update_doc.insert("email", email);
        }
        if let Some(password) = update_data.password {
            let hashed_password = bcrypt::hash(password, bcrypt::DEFAULT_COST).map_err(|e| {
                error!("Failed to hash password: {}", e);
                e.to_string()
            })?;
            update_doc.insert("password", hashed_password);
        }
        if let Some(default_asset) = update_data.default_asset {
            update_doc.insert("default_asset", default_asset);
        }

        collection
            .update_one(doc! { "_id": user_id }, doc! { "$set": update_doc })
            .await
            .map_err(|e| {
                error!("Failed to update user: {}", e);
                e.to_string()
            })?;

        info!("Updated user: {}", user_id); // Registrar información cuando se actualiza el usuario con éxito
        Self::get_user(user_id, db_context).await
    }
}
