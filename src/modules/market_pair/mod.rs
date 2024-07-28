pub mod market_pair_schema;
pub mod market_pair_service;
pub mod market_pair_controller;

use actix_web::web;

pub fn init(cfg: &mut web::ServiceConfig) {
    cfg.service(market_pair_controller::get_market_pairs_by_exchange);
    cfg.service(market_pair_controller::get_all_market_pairs_with_pagination);    
    cfg.service(market_pair_controller::create_market_pair);
    cfg.service(market_pair_controller::get_market_pair);
    cfg.service(market_pair_controller::update_market_pair);
    cfg.service(market_pair_controller::delete_market_pair);
    
}
