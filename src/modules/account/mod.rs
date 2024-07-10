pub mod account_service;
pub mod account_controller;

use actix_web::web;

pub fn init(cfg: &mut web::ServiceConfig) {
    cfg.service(account_controller::get_user);
    cfg.service(account_controller::update_user);
}
