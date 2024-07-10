use actix_web::web;

pub fn configure(cfg: &mut web::ServiceConfig) {
    cfg.configure(crate::modules::auth::init);
    cfg.configure(crate::modules::account::init);
    cfg.configure(crate::modules::asset::init); // Añadir el módulo de assets
}
