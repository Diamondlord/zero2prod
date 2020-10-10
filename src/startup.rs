use crate::routes::*;
use actix_web::dev::Server;
use actix_web::{web, App, HttpServer};
use sqlx::PgPool;
use std::net::TcpListener;
use std::sync::Arc;

// Notice the different signature!
// We return `Server` on the happy path and we dropped the `async` keyword
// We have no .await call, so it is not needed anymore.
pub fn run(listener: TcpListener, pg_pool: PgPool) -> Result<Server, std::io::Error> {
    // Wrap the connection in an Arc smart pointer
    let pg_pool = Arc::new(pg_pool);
    let server = HttpServer::new(move || {
        App::new()
            .route("/health_check", web::get().to(health_check))
            .route("/subscriptions", web::post().to(subscribe))
            // Get a pointer copy and attach it to the application state
            .data(pg_pool.clone())
    })
    .listen(listener)?
    .run();
    // No .await here!
    Ok(server)
}
