use actix_web::web;

pub fn configure(cfg: &mut web::ServiceConfig) {
    cfg.configure(crate::modules::auth::init);
    cfg.configure(crate::modules::account::init);
    cfg.configure(crate::modules::asset::init); // Añadir el módulo de assets
    cfg.configure(crate::modules::market_pair::init); // Añadir el módulo de market_pair
    cfg.configure(crate::modules::exchange::init);
    cfg.configure(crate::modules::arbitrage_strategy::init);
}
