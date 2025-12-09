use actix_web::{
    HttpResponse,
    web::{self, ServiceConfig},
};

use crate::auth::{handlers::me, init_routes as auth_routes, middleware::AuthMiddleware};

pub fn init_routes(cfg: &mut ServiceConfig) {
    cfg.service(
        web::scope("/api/v1")
            .route(
                "/health",
                web::get().to(|| async { HttpResponse::Ok().body("ok") }),
            )
            .service(web::scope("/auth").configure(auth_routes))
            .service(
                web::scope("/proc")
                    .wrap(AuthMiddleware::new())
                    .route("/me", web::get().to(me)),
            ),
    );
}
