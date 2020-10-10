use actix_web::{web, HttpResponse};
use std::sync::Arc;
use sqlx::PgPool;
use std::ops::Deref;
use chrono::Utc;
use uuid::Uuid;

#[derive(serde::Deserialize)]
pub struct FormData {
    email: String,
    name: String
}

// Let's start simple: we always return a 200 OK
pub async fn subscribe(form: web::Form<FormData>,
                       // Retrieving a connection from the application state!
                       pg_pool: web::Data<Arc<PgPool>>,
) -> Result<HttpResponse, HttpResponse> {
    sqlx::query!(
        r#"
        INSERT INTO subscriptions (id, email, name, subscribed_at)
        VALUES ($1, $2, $3, $4)
        "#,
        Uuid::new_v4(),
        form.email,
        form.name,
        Utc::now()
    )
        // There is a bit of cerenomy here to get our hands on a &PgConnection.
        // web::Data<Arc<PgConnection>> is equivalent to Arc<Arc<PgConnection>>
        // Therefore connection.get_ref() returns a &Arc<PgConnection>
        // which we can then deref to a &PgConnection.
        // We could have avoided the double Arc wrapping using .app_data()
        // instead of .data() in src/startup.rs
        .execute(pg_pool.get_ref().deref())
        .await
        .map_err(|e| {
            eprintln!("Failed to execute query: {}", e);
            HttpResponse::InternalServerError().finish()
        })?;
    Ok(HttpResponse::Ok().finish())
}
