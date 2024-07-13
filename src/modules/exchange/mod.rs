pub mod exchange_schema;
pub mod exchange_service;
pub mod exchange_controller;

use actix_web::web;

pub fn init(cfg: &mut web::ServiceConfig) {
    cfg.service(exchange_controller::create_exchange);
    cfg.service(exchange_controller::get_exchange);
    cfg.service(exchange_controller::update_exchange);
    cfg.service(exchange_controller::delete_exchange);
    cfg.service(exchange_controller::get_all_exchanges);
}
