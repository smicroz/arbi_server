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

#[derive(Serialize, Deserialize)]
pub struct AuthResponse {
    pub id: String,           // Añadir campo id
    pub token: String,
    pub name: String,
    pub email: String,
    pub default_asset: Option<String>, // Enviar como String en la respuesta JSON
}

pub struct AuthService;

impl AuthService {
    pub async fn login(email: &str, password: &str, db_context: &MongoDbContext) -> Result<AuthResponse, String> {
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

        // Configurar el tiempo de expiración del token (por ejemplo, 12 horas)
        let expiration = Utc::now() + Duration::hours(12);
        let my_claims = Claims {
            sub: user.id.unwrap().to_hex(),
            exp: expiration.timestamp() as usize,
        };
        let token = encode(&Header::default(), &my_claims, &EncodingKey::from_secret(env::var("SECRET_KEY").expect("SECRET_KEY must be set").as_ref())).map_err(|e| e.to_string())?;

        let auth_response = AuthResponse {
            id: user.id.unwrap().to_hex(), // Añadir el id del usuario
            token,
            name: user.name,
            email: user.email,
            default_asset: user.default_asset.map(|id| id.to_hex()), // Convertir ObjectId a String
        };

        Ok(auth_response)
    }

    pub async fn register(name: &str, email: &str, password: &str, db_context: &MongoDbContext) -> Result<AuthResponse, String> {
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

        let mut user = User {
            id: None,
            name: name.to_string(),
            email: email.to_string(),
            password: hashed_password,
            default_asset: None, // El campo default_asset está vacío al registrar
            password_reset_token: String::new(),
            password_reset_expires: chrono::Utc::now().naive_utc(),
            tokens: vec![],
            role: "user".to_string(),
        };

        let insert_result = collection
            .insert_one(&user)
            .await
            .map_err(|e| e.to_string())?;

        // Actualizar el user.id con el _id generado por MongoDB
        user.id = Some(insert_result.inserted_id.as_object_id().unwrap());

        // Generar el token JWT para el nuevo usuario registrado
        let expiration = Utc::now() + Duration::hours(12);
        let my_claims = Claims {
            sub: user.id.unwrap().to_hex(),
            exp: expiration.timestamp() as usize,
        };
        let token = encode(&Header::default(), &my_claims, &EncodingKey::from_secret(env::var("SECRET_KEY").expect("SECRET_KEY must be set").as_ref())).map_err(|e| e.to_string())?;

        let auth_response = AuthResponse {
            id: user.id.unwrap().to_hex(), // Añadir el id del usuario
            token,
            name: user.name,
            email: user.email,
            default_asset: user.default_asset.map(|id| id.to_hex()), // Convertir ObjectId a String
        };

        info!("User registered: {}", email);
        Ok(auth_response)
    }
}
