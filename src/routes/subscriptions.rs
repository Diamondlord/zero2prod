use actix_web::{web, HttpResponse};
use chrono::Utc;
use sqlx::PgPool;
use uuid::Uuid;
use crate::domain::{NewSubscriber, SubscriberName};

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
skip(form, db_pool),
fields(
request_id = % Uuid::new_v4(),
email = % form.email,
name = % form.name
)
)]
pub async fn subscribe(
    form: web::Form<SubscribeRequest>,
    // Retrieving a connection from the application state!
    db_pool: web::Data<PgPool>,
) -> Result<HttpResponse, HttpResponse> {
    let new_subscriber = NewSubscriber {
        email: form.0.email,
        name: SubscriberName::parse(form.0.name),
    };
    insert_subscriber(db_pool, &new_subscriber)
        .await
        .map_err(|e| {
            tracing::error!("Failed to execute query: {:?}", e);
            HttpResponse::InternalServerError().finish()
        })?;
    Ok(HttpResponse::Ok().finish())
}

#[tracing::instrument(
name = "Saving new subscriber details in the database",
skip(new_subscriber, db_pool)
)]
async fn insert_subscriber(
    db_pool: web::Data<PgPool>,
    new_subscriber: &NewSubscriber,
) -> Result<(), sqlx::Error> {
    let query = sqlx::query!(
        r#"
        INSERT INTO subscriptions (id, email, name, subscribed_at)
        VALUES ($1, $2, $3, $4)
        "#,
        Uuid::new_v4(),
        new_subscriber.email,
        new_subscriber.name.inner_ref(),
        Utc::now()
    );
    query.execute(db_pool.as_ref()).await.map_err(|e| e)?;
    Ok(())
}
