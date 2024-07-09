use actix_web::{dev::ServiceRequest, dev::ServiceResponse, Error, HttpMessage, HttpResponse};
use actix_web::dev::{Transform, Service};
use actix_web::http::StatusCode;
use actix_web::body::EitherBody;
use futures::future::{ok, Ready as FuturesReady};
use std::task::{Context, Poll};
use std::rc::Rc;
use std::cell::RefCell;
use std::pin::Pin;
use std::future::Future;
use jsonwebtoken::{decode, DecodingKey, Validation};
use serde::{Serialize, Deserialize};

#[derive(Debug, Serialize, Deserialize)]
struct Claims {
    sub: String,
    exp: usize,
}

pub struct Auth;

impl<S, B> Transform<S, ServiceRequest> for Auth
where
    S: Service<ServiceRequest, Response = ServiceResponse<B>, Error = Error> + 'static,
    S::Future: 'static,
    B: 'static,
{
    type Response = ServiceResponse<EitherBody<B>>;
    type Error = Error;
    type Transform = AuthMiddleware<S>;
    type InitError = ();
    type Future = FuturesReady<Result<Self::Transform, Self::InitError>>;

    fn new_transform(&self, service: S) -> Self::Future {
        ok(AuthMiddleware { service: Rc::new(RefCell::new(service)) })
    }
}

pub struct AuthMiddleware<S> {
    service: Rc<RefCell<S>>,
}

impl<S, B> Service<ServiceRequest> for AuthMiddleware<S>
where
    S: Service<ServiceRequest, Response = ServiceResponse<B>, Error = Error> + 'static,
    S::Future: 'static,
    B: 'static,
{
    type Response = ServiceResponse<EitherBody<B>>;
    type Error = Error;
    type Future = Pin<Box<dyn Future<Output = Result<Self::Response, Self::Error>>>>;

    fn poll_ready(&self, ctx: &mut Context<'_>) -> Poll<Result<(), Self::Error>> {
        self.service.borrow_mut().poll_ready(ctx)
    }

    fn call(&self, req: ServiceRequest) -> Self::Future {
        let svc = self.service.clone();

        Box::pin(async move {
            // Permitir acceso sin autenticación a /register y /login
            let path = req.path();
            if path == "/register" || path == "/login" {
                return Ok(svc.borrow_mut().call(req).await?.map_into_left_body());
            }

            // Verificar autenticación para otras rutas
            if let Some(authen_header) = req.headers().get("Authorization") {
                if let Ok(authen_str) = authen_header.to_str() {
                    if authen_str.starts_with("Bearer ") {
                        let token = authen_str.trim_start_matches("Bearer ").to_string();
                        let secret_key = std::env::var("SECRET_KEY").expect("SECRET_KEY must be set");
                        if let Ok(decoded) = decode::<Claims>(&token, &DecodingKey::from_secret(secret_key.as_ref()), &Validation::default()) {
                            req.extensions_mut().insert(decoded.claims);
                            return Ok(svc.borrow_mut().call(req).await?.map_into_left_body());
                        }
                    }
                }
            }

            let (req, _) = req.into_parts();
            Ok(ServiceResponse::new(
                req,
                HttpResponse::new(StatusCode::UNAUTHORIZED).map_into_right_body()
            ))
        })
    }
}
