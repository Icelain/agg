use crate::{
    llm_filter,
    sources::{HackerNews, Source},
    types::JsonResponse,
};
use axum::{Json, Router, extract::State, routing::get, serve};
use std::{sync::Arc, time::Duration};
use tokio::net::TcpListener;

use tower_http::cors::CorsLayer;

struct AppState {
    hn_source: HackerNews,
}

pub async fn run_ws(config: crate::config::Config) {
    let hn_source = HackerNews::new();
    tokio::spawn(source_refresh(hn_source.clone(), config.openai_key));
    tokio::spawn(source_clear(hn_source.clone()));

    let state = Arc::new(AppState { hn_source });
    let app = Router::new()
        .route("/", get(index_feed_handler))
        .layer(tower::ServiceBuilder::new().layer(CorsLayer::very_permissive()))
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

async fn source_refresh(mut source: impl Source, token: String) {
    let mut interval = tokio::time::interval(Duration::from_secs(60 * 60));

    loop {
        let _ = source.sync().await.inspect_err(|e| println!("error: {e}"));

        let posts = source.pull_raw().await;
        let filtered_posts = match llm_filter::filter_posts(&token, posts).await {
            Ok(posts) => posts,
            Err(e) => {
                eprintln!("Error filtering posts: {e}");
                continue;
            }
        };

        let _ = source
            .push_unconditional(filtered_posts)
            .await
            .inspect_err(|e| eprintln!("Error pushing posts: {e}"));

        interval.tick().await;
    }
}

async fn source_clear(mut source: impl Source) {
    let mut interval = tokio::time::interval(Duration::from_secs(60 * 60));

    loop {
        interval.tick().await;
        let _ = source.empty().await;
    }
}

async fn index_feed_handler(State(state): State<Arc<AppState>>) -> Json<JsonResponse> {
    let data = state.hn_source.pull().await;

    Json(JsonResponse { response: data })
}
