pub mod handlers;
pub mod jwt;
pub mod middleware;
pub mod password;

use actix_web::web;

use crate::auth::handlers::{login, logout, refresh_fresh, signup};

pub fn init_routes(cfg: &mut web::ServiceConfig) {
  cfg.service(web::resource("/signup").route(web::post().to(signup)));
  cfg.service(web::resource("/login").route(web::post().to(login)));
  cfg.service(web::resource("/refresh").route(web::post().to(refresh_fresh)));
  cfg.service(web::resource("/logout").route(web::post().to(logout)));
}
