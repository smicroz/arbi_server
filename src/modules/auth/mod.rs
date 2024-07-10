pub mod auth_service;
pub mod auth_controller;
pub mod auth_model;
pub mod auth_response;

use actix_web::web;

pub fn init(cfg: &mut web::ServiceConfig) {
    cfg.service(auth_controller::login);
    cfg.service(auth_controller::register);
}
