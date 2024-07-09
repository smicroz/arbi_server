use actix_web::web;

pub fn configure(cfg: &mut web::ServiceConfig) {
    cfg.configure(crate::modules::auth::init);
    //cfg.configure(crate::modules::user::init);
    // Añadir otros módulos según sea necesario
    // cfg.configure(crate::modules::asset::init);
}
