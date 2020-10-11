use actix_web::{web, HttpResponse};
use chrono::Utc;
use sqlx::PgPool;
use uuid::Uuid;

#[derive(serde::Deserialize)]
pub struct FormData {
    email: String,
    name: String,
}

pub async fn subscribe(
    payload: web::Form<FormData>,
    // Retrieving a connection from the application state!
    pg_pool: web::Data<PgPool>,
) -> Result<HttpResponse, HttpResponse> {
    let request_id = Uuid::new_v4();
    // Spans, like logs, have an associated level
    // `info_span` creates a span at the info-level
    let request_span = tracing::info_span!(
        "Adding a new subscriber.",
        %request_id,
        email = %payload.email,
        name = %payload.name
    );
    // Using `enter` in an async function is a recipe for disaster!
    // Bear with me for now, but don't do this at home.
    // See the following section on `tracing-futures`
    let _request_span_guard = request_span.enter();
    tracing::info!(
        "request_id {} - Saving new subscriber details in the database",
        request_id
    );
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
    .execute(pg_pool.as_ref())
    .await
    .map_err(|e| {
        log::error!("Failed to execute query: {:?}", e);
        HttpResponse::InternalServerError().finish()
    })?;
    tracing::info!(
        "request_id {} - New subscriber details have been saved",
        request_id
    );
    Ok(HttpResponse::Ok().finish())
    // `_request_span_guard` is dropped at the end of `subscribe`
    // That's when we "exit" the span
}
