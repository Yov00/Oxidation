#![allow(unused)]

use axum::{
    Router,
    extract::State,
    response::Html,
    routing::{get, get_service},
};
use sqlx::{Sqlite, SqlitePool};
use std::{
    net::{IpAddr, Ipv4Addr, SocketAddr},
    path::Path,
};
use tokio::{
    fs::File,
    io::{self, AsyncReadExt},
    net::TcpListener,
};
use tower_http::services::ServeDir;

#[tokio::main]

async fn main() {
    let pool = SqlitePool::connect("sqlite:data.db").await.unwrap();

    let routes_hello = Router::new()
        .route("/", get(home_handler))
        .route("/about", get(about_handler))
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

    let mut html: String = String::from(
        "
            <!DOCTYPE html>
            <html >
                <head>
                <link rel='stylesheet' href='/assets/bootstrap/css/bootstrap.min.css'>
                </head>

            <body>
    ",
    );

    for user in users {
        let user_content = format!(
            r#"
        <h1  class='btn btn-primary' >{}</h1>
        "#,
            user.name
        );

        html.push_str(&user_content)
    }

    html.push_str(
        r#"
    </body>
    </html>
    "#,
    );
    Html(html)
}

async fn read_html_from_file<P: AsRef<Path>>(path: P) -> io::Result<String> {
    let mut file = File::open(path).await?;
    let mut contents = String::new();
    file.read_to_string(&mut contents).await?;

    Ok(contents)
}

#[derive(sqlx::FromRow, Debug)]
struct User {
    id: i64,
    name: String,
}
