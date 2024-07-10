pub mod asset_controller;
pub mod asset_service;
pub mod asset_schema;

use actix_web::web;

pub fn init(cfg: &mut web::ServiceConfig) {
    cfg.service(asset_controller::create_asset);
    cfg.service(asset_controller::get_asset);
    cfg.service(asset_controller::update_asset);
    cfg.service(asset_controller::delete_asset);
    cfg.service(asset_controller::get_all_assets);
}
