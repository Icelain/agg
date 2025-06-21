use crate::sources::{HackerNews, Source};
use axum::{Router, extract::State, routing::get, serve};
use std::{sync::Arc, time::Duration};
use tokio::net::TcpListener;

struct AppState {
    hn_source: HackerNews,
}

pub async fn run_ws(config: crate::config::Config) {
    let hn_source = HackerNews::new();
    tokio::spawn(source_refresh(hn_source.clone()));

    let state = Arc::new(AppState { hn_source });
    let app = Router::new()
        .route("/", get(index_feed_handler))
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
        let _ = source.sync().await.inspect_err(|e| println!("error: {e}"));
        interval.tick().await;
    }
}

async fn index_feed_handler(State(state): State<Arc<AppState>>) -> String {
    let data = state.hn_source.pull().await;

    let mut response = String::new();
    for item in data {
        response.push_str(&item.title);
    }

    response
}
