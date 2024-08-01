pub mod arbitrage_strategy_schema;
pub mod arbitrage_strategy_service;
pub mod arbitrage_strategy_controller;
pub mod suggested_arbitrage_strategy_service;
pub mod suggested_arbitrage_strategy_controller;

use actix_web::web;

pub fn init(cfg: &mut web::ServiceConfig) {
    cfg.service(suggested_arbitrage_strategy_controller::get_suggested_strategies);
    cfg.service(arbitrage_strategy_controller::create_arbitrage_strategy);
    cfg.service(arbitrage_strategy_controller::get_arbitrage_strategy);
    cfg.service(arbitrage_strategy_controller::update_arbitrage_strategy);
    cfg.service(arbitrage_strategy_controller::delete_arbitrage_strategy);
    cfg.service(arbitrage_strategy_controller::get_all_arbitrage_strategies);
    
}