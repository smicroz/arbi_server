use crate::db::mongodb::MongoDbContext;
use mongodb::bson::{doc, oid::ObjectId};
use crate::modules::asset::asset_schema::Asset;
use tracing::error; // Mantenemos la importación de error
use chrono::Utc;
use futures::TryStreamExt;

pub struct AssetService;

impl AssetService {
    pub async fn create_asset(asset: Asset, db_context: &MongoDbContext) -> Result<Asset, String> {
        let db = db_context.get_database();
        let collection = db.collection::<Asset>("assets");

        let now = Utc::now().timestamp();
        let new_asset = Asset {
            created_at: now,
            updated_at: now,
            ..asset
        };

        let insert_result = collection.insert_one(new_asset.clone()).await
            .map_err(|e| {
                error!("Failed to insert asset: {}", e);
                e.to_string()
            })?;
        
        let new_asset = collection.find_one(doc! { "_id": insert_result.inserted_id }).await
            .map_err(|e| {
                error!("Failed to fetch created asset: {}", e);
                e.to_string()
            })?
            .ok_or_else(|| {
                let msg = "Failed to fetch created asset".to_string();
                error!("{}", msg);
                msg
            })?;
        
        Ok(new_asset)
    }

    pub async fn get_asset(id: ObjectId, db_context: &MongoDbContext) -> Result<Asset, String> {
        let db = db_context.get_database();
        let collection = db.collection::<Asset>("assets");

        let asset = collection.find_one(doc! { "_id": id }).await
            .map_err(|e| {
                error!("Failed to fetch asset: {}", e);
                e.to_string()
            })?
            .ok_or_else(|| {
                let msg = "Asset not found".to_string();
                error!("{}", msg);
                msg
            })?;

        Ok(asset)
    }

    pub async fn update_asset(id: ObjectId, updated_asset: Asset, db_context: &MongoDbContext) -> Result<Asset, String> {
        let db = db_context.get_database();
        let collection = db.collection::<Asset>("assets");

        let now = Utc::now().timestamp();
        let update_doc = doc! {
            "$set": {
                "name": updated_asset.name,
                "short_name": updated_asset.short_name,
                "updated_at": now,
                "status": updated_asset.status,
                "_exchange": updated_asset._exchange,
            }
        };

        collection.update_one(doc! { "_id": id }, update_doc).await
            .map_err(|e| {
                error!("Failed to update asset: {}", e);
                e.to_string()
            })?;

        Self::get_asset(id, db_context).await
    }

    pub async fn delete_asset(id: ObjectId, db_context: &MongoDbContext) -> Result<(), String> {
        let db = db_context.get_database();
        let collection = db.collection::<Asset>("assets");

        collection.delete_one(doc! { "_id": id }).await
            .map_err(|e| {
                error!("Failed to delete asset: {}", e);
                e.to_string()
            })?;

        Ok(())
    }

    pub async fn get_all_assets(db_context: &MongoDbContext) -> Result<Vec<Asset>, String> {
        let db = db_context.get_database();
        let collection = db.collection::<Asset>("assets");

        let mut cursor = collection.find(doc! {}).await
            .map_err(|e| {
                error!("Failed to fetch all assets: {}", e);
                e.to_string()
            })?;

        
        let mut assets = Vec::new();
        while let Some(asset) = cursor.try_next().await.map_err(|e| {
            error!("Failed to iterate through assets: {}", e);
            e.to_string()
        })? {
            assets.push(asset);
        }

        //println!("assets: {:?}", assets);

        Ok(assets)
    }
}
