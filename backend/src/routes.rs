use actix_web::{HttpResponse, web::{self, ServiceConfig}};

use crate::auth::{handlers::me, init_routes, middleware::AuthMiddleware};

pub fn init_routes(cfg: &mut web::ServiceConfig){
  web::service(
    web::scope("/api/v1")
              .service(web::scope("/auth").configure(init_routes))
              .service(web::scope("").wrap(AuthMiddleware::new()).route("/me", web::get().to(me)),
            )
            .route("/health", web::get().to(|| async {HttpResponse::Ok().body("ok")}))
  )
} 
