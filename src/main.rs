use axum::{
    extract::Path,
    routing::get,
    Router,
};

async fn hello_name(Path(name): Path<String>) -> String {
    format!("Hello {}", name)
}

#[tokio::main]
async fn main() {
    let app = Router::new()
        .route("/hello/{name}", get(hello_name));

    let listener = tokio::net::TcpListener::bind("0.0.0.0:3000")
        .await
        .unwrap();

    axum::serve(listener, app)
        .await
        .unwrap();
}