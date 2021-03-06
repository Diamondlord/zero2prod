use crate::domain::{NewSubscriber, SubscriberEmail, SubscriberName};
use crate::email_client::EmailClient;
use actix_web::{web, HttpResponse};
use chrono::Utc;
use sqlx::PgPool;
use std::convert::TryInto;
use uuid::Uuid;

#[derive(serde::Deserialize)]
pub struct SubscribeRequest {
    email: String,
    name: String,
}

impl TryInto<NewSubscriber> for SubscribeRequest {
    type Error = String;

    fn try_into(self) -> Result<NewSubscriber, Self::Error> {
        let name = SubscriberName::parse(self.name)?;
        let email = SubscriberEmail::parse(self.email)?;
        Ok(NewSubscriber { email, name })
    }
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
    email_client: web::Data<EmailClient>,
) -> Result<HttpResponse, HttpResponse> {
    let new_subscriber = form
        .0
        .try_into()
        .map_err(|_| HttpResponse::BadRequest().finish())?;
    insert_subscriber(db_pool, &new_subscriber)
        .await
        .map_err(|e| {
            tracing::error!("Failed to execute query: {:?}", e);
            HttpResponse::InternalServerError().finish()
        })?;

    send_confirmation_email(email_client, new_subscriber)
        .await
        .map_err(|e| {
            tracing::error!("failed to send confirmation email: {:?}", e);
            HttpResponse::InternalServerError().finish()
        });

    Ok(HttpResponse::Ok().finish())
}

#[tracing::instrument(
    name = "Saving new subscriber details in the database",
    skip(db_pool, new_subscriber)
)]
async fn insert_subscriber(
    db_pool: web::Data<PgPool>,
    new_subscriber: &NewSubscriber,
) -> Result<(), sqlx::Error> {
    let query = sqlx::query!(
        r#"
        INSERT INTO subscriptions (id, email, name, subscribed_at, status)
        VALUES ($1, $2, $3, $4, 'confirmed')
        "#,
        Uuid::new_v4(),
        new_subscriber.email.as_ref(),
        new_subscriber.name.as_ref(),
        Utc::now()
    );
    query.execute(db_pool.as_ref()).await?;
    Ok(())
}

#[tracing::instrument(
    name = "sending confirmation email",
    skip(email_client, new_subscriber)
)]
async fn send_confirmation_email(
    email_client: web::Data<EmailClient>,
    new_subscriber: NewSubscriber,
) -> Result<(), reqwest::Error> {
    let confirmation_link = "https://my-api.com/subscriptions/confirm";
    let plain_body = format!(
        "Welcome to our newsletter!\nVisit {} to confirm your subscription.",
        confirmation_link
    );
    let html_body = format!(
        "Welcome to our newsletter!<br />\
        Click <a href=\"{}\">here</a> to confirm your subscription.",
        confirmation_link
    );
    email_client
        .send_email(new_subscriber.email, "Welcome!", &html_body, &plain_body)
        .await
}
