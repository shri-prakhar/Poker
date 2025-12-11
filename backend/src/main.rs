use crate::{config::Setting, routes::init_routes, state::AppState, telemetry::init_tracing};
use actix_web::{App, HttpServer, middleware::Logger, web};
use anyhow::Ok;
use database::create_pool;
use tracing::info;

mod auth;
mod config;
mod errors;
mod game_manager;
mod poker_engine;
mod routes;
mod state;
mod telemetry;
mod ws_server;

#[actix_web::main]
async fn main() -> anyhow::Result<()> {
    let _ = init_tracing();

    let setting = Setting::from_env()?;
    info!(
        "Starting Server on: {} , with workers: {}",
        &setting.bind_addr, &setting.worker_threads
    );

    let pool = create_pool(&setting.database_url).await?;

    let app_state = AppState::new(pool.clone(), setting.clone()).await?;
    let app_data = web::Data::new(app_state.clone());

    let server = HttpServer::new(move || {
        App::new()
            .wrap(Logger::default())
            .app_data(app_data.clone())
            .configure(init_routes)
    })
    .workers(setting.worker_threads)
    .bind(&setting.bind_addr)?
    .shutdown_timeout(30)
    .run();

    let server_handle = server.handle();
    let shutdown_listener = tokio::spawn(async move {
        if tokio::signal::ctrl_c().await.is_ok() {
            tracing::info!("ShutDown signal recieved : Stopping the server");
            server_handle.stop(true).await;
        }
    });

    server.await?;
    let _ = shutdown_listener.await?;
    info!("!!Server Stopped Cleanly");
    Ok(())
}
