use crate::modules::user::user_schema::User;
use mongodb::bson::{doc};
use bcrypt::{hash, verify, DEFAULT_COST}; // Importar bcrypt
use jsonwebtoken::{encode, Header, EncodingKey}; // Importaciones necesarias
use tracing::{info, error};
use crate::db::mongodb::MongoDbContext;
use chrono::{Utc, Duration};
use std::env;
use serde::{Serialize, Deserialize};

#[derive(Debug, Serialize, Deserialize)]
struct Claims {
    sub: String,
    exp: usize,
}

pub struct AuthService;

impl AuthService {
    pub async fn login(email: &str, password: &str, db_context: &MongoDbContext) -> Result<String, String> {
        let db = db_context.get_database();
        let collection = db.collection::<User>("users");

        // Buscar el usuario por email
        let user = collection
            .find_one(doc! { "email": email })
            .await
            .map_err(|e| e.to_string())?
            .ok_or_else(|| {
                error!("User not found: {}", email);
                "User not found".to_string()
            })?;

        // Verificar la contraseña proporcionada con el hash almacenado
        if !verify(password, &user.password).map_err(|e| e.to_string())? {
            error!("Invalid password for user: {}", email);
            return Err("Invalid password".to_string());
        }

        // Configurar el tiempo de expiración del token (por ejemplo, 12 hora)
        let expiration = Utc::now() + Duration::hours(12);
        let my_claims = Claims {
            sub: user.id.unwrap().to_hex(),
            exp: expiration.timestamp() as usize,
        };
        let token = encode(&Header::default(), &my_claims, &EncodingKey::from_secret(env::var("SECRET_KEY").expect("SECRET_KEY must be set").as_ref())).map_err(|e| e.to_string())?;

        Ok(token)
    }

    pub async fn register(name: &str, email: &str, password: &str, db_context: &MongoDbContext) -> Result<User, String> {
        let db = db_context.get_database();
        let collection = db.collection::<User>("users");

        // Verificar si el usuario ya existe
        if let Some(_) = collection
            .find_one(doc! { "email": email })
            .await
            .map_err(|e| e.to_string())? {
            return Err("User already exists".to_string());
        }

        // Generar hash de la contraseña
        let hashed_password = hash(password, DEFAULT_COST).map_err(|e| e.to_string())?;

        let user = User {
            id: None,
            name: name.to_string(),
            email: email.to_string(),
            password: hashed_password,
            password_reset_token: String::new(),
            password_reset_expires: chrono::Utc::now().naive_utc(),
            tokens: vec![],
            role: "user".to_string(),
        };

        collection
            .insert_one(user.clone())
            .await
            .map_err(|e| e.to_string())?;

        info!("User registered: {}", email);
        Ok(user)
    }
}
