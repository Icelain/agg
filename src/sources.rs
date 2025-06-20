use serde::{Deserialize, Serialize};
use serde_json::Value;

use std::sync::{Arc, Mutex};

const HN_TOP_URL: &str = "https://hacker-news.firebaseio.com/v0/topstories.json";
const HN_POST_URL: &str = "https://hacker-news.firebaseio.com/v0/item/";

#[derive(Serialize, Deserialize, Default, Clone)]
pub struct Post {
    pub title: String,
    pub url: Option<String>,
}

pub trait Source {
    async fn sync(&mut self) -> Result<(), anyhow::Error>;
    async fn pull(&self) -> Vec<Post>;
}

#[derive(Clone)]
pub struct HackerNews {
    posts: Arc<Mutex<Vec<Post>>>,
}

impl Source for HackerNews {
    async fn sync(&mut self) -> Result<(), anyhow::Error> {
        let req = match reqwest::get(HN_TOP_URL).await {
            Ok(req) => req,
            Err(e) => {
                return Err(anyhow::anyhow!(
                    "Error while pulling from hackernews: {}",
                    e
                ));
            }
        };

        let req_value: Value = serde_json::from_str(req.text().await.unwrap().as_str()).unwrap();
        let top_story_ids = req_value
            .as_array()
            .unwrap()
            .iter()
            .map(|v| v.as_u64().unwrap());

        self.hit(top_story_ids).await;

        Ok(())
    }

    async fn pull(&self) -> Vec<Post> {
        let post_c = self.posts.clone();
        let guard = post_c.lock().unwrap();

        let current_state = guard.to_vec();
        drop(guard);

        current_state
    }
}

impl HackerNews {
    pub(crate) fn new() -> HackerNews {
        HackerNews {
            posts: Arc::new(Mutex::new(Vec::with_capacity(50))),
        }
    }

    async fn hit(&mut self, story_ids: impl Iterator<Item = u64>) {
        for id in story_ids {
            let mut post_url = HN_POST_URL.to_string();
            post_url.push_str(&id.to_string());
            post_url.push_str(".json");

            let posts_c = self.posts.clone();
            tokio::spawn(async move {
                let resp = match reqwest::get(post_url).await {
                    Ok(raw_resp) => raw_resp.text().await.unwrap(),
                    Err(_) => return,
                };
                let post: Post = serde_json::from_str(&resp).unwrap();
                let mut guard = posts_c.lock().unwrap();

                if guard.len() >= 50 {
                    guard.resize_with(49, Default::default);
                }

                if post.url.clone().is_some() {
                    println!("{}:{}", post.title, post.url.clone().unwrap());
                }
                guard.push(post);

                drop(guard);
            });
        }
    }
}
