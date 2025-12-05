pub mod handlers;
pub mod jwt;
pub mod middleware;
pub mod password;

use actix_web::web;

pub fn init_routes(cfg: &mut web::ServiceConfig) {}
