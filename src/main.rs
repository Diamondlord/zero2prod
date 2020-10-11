use sqlx::PgPool;
use std::net::TcpListener;
use zero2prod::configuration::get_configuration;
use zero2prod::startup::run;
use env_logger::Env;

#[actix_rt::main]
async fn main() -> std::io::Result<()> {
    // `init` does call `set_logger`, so this is all we need to do.
    // We are falling back to printing all logs at info-level or above
    // if the RUST_LOG environment variable has not been set.
    env_logger::from_env(Env::default().default_filter_or("info")).init();
    let configuration = get_configuration().expect("Failed to read configuration.");
    let connection_string = configuration.database.connection_string();
    let pg_pool = PgPool::connect(&connection_string)
        .await
        .expect("Failed to connect to Postgres.");
    let address = format!("127.0.0.1:{}", configuration.application_port);
    let listener = TcpListener::bind(address)?;
    run(listener, pg_pool)?.await
}
