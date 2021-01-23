use crate::routes::*;
use actix_web::dev::Server;
use actix_web::web::Data;
use actix_web::{web, App, HttpServer};
use actix_web_opentelemetry::RequestTracing;
use sqlx::PgPool;
use std::net::TcpListener;
use tracing_actix_web::TracingLogger;
use crate::email_client::EmailClient;

pub fn run(listener: TcpListener, pg_pool: PgPool, email_client: EmailClient) -> Result<Server, std::io::Error> {
    let pg_pool = Data::new(pg_pool);
    let email_client = Data::new(email_client);
    let server = HttpServer::new(move || {
        App::new()
            .wrap(TracingLogger)
            .wrap(RequestTracing::new())
            .route("/health_check", web::to(health_check))
            .route("/subscriptions", web::post().to(subscribe))
            // Get a pointer copy and attach it to the application state
            .app_data(pg_pool.clone())
    })
    .listen(listener)?
    .run();
    // No .await here!
    Ok(server)
}
