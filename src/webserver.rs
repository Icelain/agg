use crate::sources::{HackerNews, Source};
use axum::{Router, extract::State, routing::get, serve};
use std::{sync::Arc, time::Duration};
use tokio::net::TcpListener;

struct AppState {
    hn_source: HackerNews,
}

pub async fn run_ws(config: crate::config::Config) {
    let mut hn_source = HackerNews::new();
    tokio::spawn(source_refresh(hn_source.clone()));

    let state = Arc::new(AppState { hn_source });
    let app = Router::new()
        .route("/", get(|| async { ";)" }))
        .with_state(state)
        .into_make_service();

    let listener = if config.webserver_address.is_some() {
        TcpListener::bind(config.webserver_address.unwrap())
            .await
            .unwrap()
    } else {
        TcpListener::bind(format!("0.0.0.0:{}", config.webserver_port.unwrap()))
            .await
            .unwrap()
    };

    serve(listener, app).await.unwrap();
}

async fn source_refresh(mut source: impl Source) {
    let mut interval = tokio::time::interval(Duration::from_secs(60));

    loop {
        source.sync().await.inspect_err(|e| println!("error: {e}"));
        interval.tick().await;
    }
}
