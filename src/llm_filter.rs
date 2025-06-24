use axum::extract;
use reqwest::Client;
use serde_json::json;

use crate::{sources::Post, types::JsonResponse};

const OPENAI_API_URL: &str = "https://api.openai.com/v1/responses";

pub async fn filter_posts(
    openai_token: &str,
    in_posts: Vec<Post>,
) -> Result<Vec<Post>, anyhow::Error> {
    if in_posts.is_empty() {
        return Err(anyhow::anyhow!("Empty posts vector"));
    }

    let client = Client::new();
    let mut in_posts_string = String::new();

    in_posts.iter().for_each(|post| {
        in_posts_string.push_str(&post.title);
        in_posts_string.push_str("<::>");

        // post urls are guaranteed to be Some(String) here
        let url = post.url.as_ref().unwrap();
        in_posts_string.push_str(&url);
        in_posts_string.push('\n');
    });

    let mut ch = in_posts_string.chars();
    ch.next_back();

    in_posts_string = ch.as_str().to_string();

    let filter_prompt_json = json! ({
    "model": "gpt-4.1",
    "input": format!("Take the given entries detailing tech articles and only keep the entries that are related to AI and ML. ONLY OUTPUT AS THEY ARE PROVIDED, NOTHING ELSE, NOT EVEN MARKDOWN BACKTICK INDICATORS; USE \n to seperate entries: {}", in_posts_string)
    });

    let filtered_json_str = client
        .post(OPENAI_API_URL)
        .header("Content-Type", "application/json")
        .bearer_auth(openai_token)
        .body(serde_json::to_string(&filter_prompt_json)?)
        .send()
        .await?
        .text()
        .await?;

    let filtered_json_response: serde_json::Value = serde_json::from_str(&filtered_json_str)?;

    let extracted_json_response =
        serde_json::to_string(&filtered_json_response["output"][0]["content"][0]["text"])?;

    // extracted_json_response = extracted_json_response
    //     .replace("\n", "")
    //     .replace("\\n", "")
    //     .replace("\\", "")

    // let mut chars = extracted_json_response.chars();
    // chars.next();
    // chars.next_back();
    // let final_json_response = chars.as_str();

    let mut out_posts: Vec<Post> = Vec::new();

    extracted_json_response.split("\\n").for_each(|line| {
        let mut linesplit = line.split("<::>");

        let title = linesplit.next().unwrap();
        let url = linesplit.next().unwrap();

        out_posts.push(Post {
            title: title.to_string(),
            url: Some(url.to_string()),
        })
    });

    Ok(out_posts)
}
