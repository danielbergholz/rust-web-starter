use std::str::FromStr;

use askama::Template;
use axum::{
    Form, Router,
    extract::{Path, Query, State},
    http::StatusCode,
    response::{Html, IntoResponse, Redirect, Response},
    routing::{get, post},
};
use serde::Deserialize;
use sqlx::{FromRow, SqlitePool, sqlite::SqliteConnectOptions};
use tower_http::{services::ServeDir, trace::TraceLayer};
use tracing_subscriber::EnvFilter;

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    // Log level comes from RUST_LOG (default: info, plus one line per request).
    tracing_subscriber::fmt()
        .with_env_filter(
            EnvFilter::try_from_default_env()
                .unwrap_or_else(|_| EnvFilter::new("info,tower_http=debug")),
        )
        .init();

    // Same variable sqlx-cli reads, so `sqlx migrate ...` targets the same file.
    let database_url = env_or("DATABASE_URL", "sqlite:app.db");
    let options = SqliteConnectOptions::from_str(&database_url)?.create_if_missing(true);
    let db = SqlitePool::connect_with(options).await?;

    // Migrations from ./migrations are embedded at compile time and applied on startup.
    sqlx::migrate!().run(&db).await?;

    let addr = env_or("ADDR", "127.0.0.1:3000");
    let listener = tokio::net::TcpListener::bind(&addr).await?;
    tracing::info!("listening on http://{addr}");
    axum::serve(listener, app(db)).await?;

    Ok(())
}

fn app(db: SqlitePool) -> Router {
    Router::new()
        .route("/", get(home))
        .route("/notes", post(create_note))
        .route("/notes/search", get(search_notes))
        .route("/notes/{id}/delete", post(delete_note))
        .nest_service("/static", ServeDir::new("static"))
        .layer(TraceLayer::new_for_http())
        .with_state(db)
}

fn env_or(key: &str, default: &str) -> String {
    std::env::var(key).unwrap_or_else(|_| default.to_string())
}

#[derive(FromRow)]
struct Note {
    id: i64,
    text: String,
    created_at: String,
}

// --- Templates ---------------------------------------------------------------

#[derive(Template)]
#[template(path = "home.html")]
struct HomeTemplate {
    notes: Vec<Note>,
    error: Option<String>,
}

// Only the list items, used by the live search to swap part of the page.
#[derive(Template)]
#[template(path = "notes.html")]
struct NotesTemplate {
    notes: Vec<Note>,
}

// --- Handlers ----------------------------------------------------------------

async fn home(State(db): State<SqlitePool>) -> Result<Html<String>, AppError> {
    let notes = find_notes(&db, "").await?;

    Ok(Html(HomeTemplate { notes, error: None }.render()?))
}

#[derive(Deserialize)]
struct SearchParams {
    #[serde(default)]
    q: String,
}

async fn search_notes(
    State(db): State<SqlitePool>,
    Query(params): Query<SearchParams>,
) -> Result<Html<String>, AppError> {
    let notes = find_notes(&db, params.q.trim()).await?;

    Ok(Html(NotesTemplate { notes }.render()?))
}

#[derive(Deserialize)]
struct NewNote {
    text: String,
}

async fn create_note(
    State(db): State<SqlitePool>,
    Form(new_note): Form<NewNote>,
) -> Result<Response, AppError> {
    // Always validate on the server: browser-side checks are easy to bypass.
    let text = match validate_note(&new_note.text) {
        Ok(text) => text,
        Err(message) => return error_page(&db, message).await,
    };

    sqlx::query("INSERT INTO notes (text) VALUES (?)")
        .bind(text)
        .execute(&db)
        .await?;

    // Post/Redirect/Get: reloading the page does not submit the form again.
    Ok(Redirect::to("/").into_response())
}

async fn delete_note(
    State(db): State<SqlitePool>,
    Path(id): Path<i64>,
) -> Result<Redirect, AppError> {
    sqlx::query("DELETE FROM notes WHERE id = ?")
        .bind(id)
        .execute(&db)
        .await?;

    Ok(Redirect::to("/"))
}

// --- Helpers -----------------------------------------------------------------

const MAX_NOTE_LENGTH: usize = 500;

fn validate_note(text: &str) -> Result<&str, &'static str> {
    let text = text.trim();

    if text.is_empty() {
        return Err("A note cannot be empty.");
    }

    if text.chars().count() > MAX_NOTE_LENGTH {
        return Err("A note can have at most 500 characters.");
    }

    Ok(text)
}

async fn find_notes(db: &SqlitePool, query: &str) -> Result<Vec<Note>, sqlx::Error> {
    // SQLite's LIKE is case-insensitive for ASCII.
    sqlx::query_as("SELECT id, text, created_at FROM notes WHERE text LIKE ? ORDER BY id DESC")
        .bind(format!("%{query}%"))
        .fetch_all(db)
        .await
}

// Renders the home page again with the message at the top.
async fn error_page(db: &SqlitePool, message: &str) -> Result<Response, AppError> {
    let page = HomeTemplate {
        notes: find_notes(db, "").await?,
        error: Some(message.to_string()),
    };

    Ok((StatusCode::BAD_REQUEST, Html(page.render()?)).into_response())
}

// Any unexpected error becomes a 500 and is logged with its cause.
struct AppError(Box<dyn std::error::Error + Send + Sync>);

impl<E: Into<Box<dyn std::error::Error + Send + Sync>>> From<E> for AppError {
    fn from(err: E) -> Self {
        Self(err.into())
    }
}

impl IntoResponse for AppError {
    fn into_response(self) -> Response {
        tracing::error!("{}", self.0);
        (StatusCode::INTERNAL_SERVER_ERROR, "Something went wrong.").into_response()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn trims_valid_notes() {
        assert_eq!(validate_note("  buy milk  "), Ok("buy milk"));
    }

    #[test]
    fn rejects_empty_notes() {
        assert!(validate_note("   ").is_err());
    }

    #[test]
    fn rejects_notes_that_are_too_long() {
        assert!(validate_note(&"a".repeat(MAX_NOTE_LENGTH)).is_ok());
        assert!(validate_note(&"a".repeat(MAX_NOTE_LENGTH + 1)).is_err());
    }
}
