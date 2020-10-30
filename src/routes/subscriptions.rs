use actix_web::{web, HttpResponse};
use chrono::Utc;
use sqlx::PgPool;
use tracing_futures::Instrument;
use uuid::Uuid;

#[derive(serde::Deserialize)]
pub struct SubscribeRequest {
    email: String,
    name: String,
}

/**
#[tracing_instrument] create a span at the beginning of the function invocation.

#[tracing_instrument] automatically attaches all arguments passed to the function
to the context of the span - in our case, payload and pool.
Often function arguments won't be displayable on log records (e.g. pool)
or we'd like to specify more explicitly what should/how they should be captured
(e.g. naming each field of payload) - we can explicitly tell tracing to ignore them using the skip directive.
*/
#[tracing::instrument(
    name = "Adding a new subscriber",
    skip(payload, db_pool),
    fields(
        request_id=%Uuid::new_v4(),
        email = %payload.email,
        name = %payload.name
    )
)]
pub async fn subscribe(
    payload: web::Form<SubscribeRequest>,
    // Retrieving a connection from the application state!
    db_pool: web::Data<PgPool>,
) -> Result<HttpResponse, HttpResponse> {
    insert_subscriber(db_pool, payload)
        .await
        .map_err(|e| HttpResponse::InternalServerError().finish())?;
    Ok(HttpResponse::Ok().finish())
}

#[tracing::instrument(
    name = "Saving new subscriber details in the database",
    skip(payload, db_pool)
)]
async fn insert_subscriber(
    db_pool: web::Data<PgPool>,
    payload: web::Form<SubscribeRequest>,
) -> Result<(), sqlx::Error> {
    sqlx::query!(
        r#"
        INSERT INTO subscriptions (id, email, name, subscribed_at)
        VALUES ($1, $2, $3, $4)
        "#,
        Uuid::new_v4(),
        payload.email,
        payload.name,
        Utc::now()
    )
    .execute(db_pool.as_ref())
    .await
    .map_err(|e| {
        tracing::error!("Failed to execute query: {:?}", e);
        e
    })?;
    Ok(())
}
