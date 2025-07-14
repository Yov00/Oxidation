#![allow(unused)]

use axum::{
    Json, Router,
    extract::{Multipart, State},
    response::{Html, IntoResponse, Response},
    routing::{get, get_service, post},
};
use serde::Serialize;
use sqlx::{Sqlite, SqlitePool};
use std::{
    iter,
    net::{IpAddr, Ipv4Addr, SocketAddr},
    ops::Mul,
    path::Path,
};
use tokio::{
    fs::File,
    io::{self, AsyncReadExt},
    net::TcpListener,
};
use tower_http::services::ServeDir;
// Types mayps
#[derive(sqlx::FromRow, Debug, Serialize)]
struct User {
    id: i64,
    name: String,
}

#[tokio::main]
async fn main() {
    let pool = SqlitePool::connect("sqlite:data.db").await.unwrap();

    let routes_hello = Router::new()
        .route("/", get(home_handler))
        .route("/about", get(about_handler))
        .route("/api/getUsers", get(handle_users))
        .route("/api/wav-to-mp3", post(handle_wav_to_mp3))
        .nest_service(
            "/assets",
            get_service(ServeDir::new("static/assets")).handle_error(|e| async move {
                (
                    axum::http::StatusCode::INTERNAL_SERVER_ERROR,
                    format!("Static file error: {}", e),
                )
            }),
        )
        .with_state(pool.clone());

    let addr = SocketAddr::from(([127, 0, 0, 1], 8080));
    let listener = TcpListener::bind("127.0.0.1:8080").await.unwrap();

    println!("->> LISTENING on {:?}\n", listener.local_addr());
    axum::serve(listener, routes_hello).await.unwrap();
}

async fn home_handler() -> Html<String> {
    let html_content = match read_html_from_file("static/index.html").await {
        Ok(html) => html,
        Err(e) => {
            eprintln!("Failed to read HTML file: {e}");
            "<h1>Error loading HTML file</h1>".to_string()
        }
    };
    Html(html_content)
}

async fn about_handler(State(pool): State<SqlitePool>) -> Html<String> {
    let html_content = match read_html_from_file("static/about.html").await {
        Ok(html) => html,
        Err(e) => {
            eprint!("Failed to read HTML file: {e}");
            "<h1>Error loading the About page".to_string()
        }
    };
    Html(html_content)
}

async fn handle_users(State(pool): State<SqlitePool>) -> Json<Vec<User>> {
    let users: Vec<User> = match sqlx::query_as::<_, User>("SELECT id, name FROM users")
        .fetch_all(&pool)
        .await
    {
        Ok(users) => users,
        Err(e) => {
            eprintln!("Failed to fetch users:{}", e);
            vec![]
        }
    };

    Json(users)
}

async fn handle_wav_to_mp3(mut multipart: Multipart) -> Response {
    let mut result = String::from("");

    let mut iteration = 0;
    while let Some(field_result) = multipart.next_field().await.unwrap_or(None) {
        let file_name = field_result
            .file_name()
            .map(|s| s.to_string())
            .unwrap_or_default();

        match Some(file_name) {
            Some(a) => {
                result = a;
                println!("There is SOME: {result}");
            }
            None => {
                break;
            }
        }
    }

    println!("aaand Returned");

    (axum::http::StatusCode::OK, result).into_response()
}

async fn read_html_from_file<P: AsRef<Path>>(path: P) -> io::Result<String> {
    let mut file = File::open(path).await?;
    let mut contents = String::new();
    file.read_to_string(&mut contents).await?;

    Ok(contents)
}
