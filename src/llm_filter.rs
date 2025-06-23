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
    let json_content = serde_json::to_string_pretty(&JsonResponse { response: in_posts })?;

    let filter_prompt_json = json! ({
    "model": "gpt-4.1",
    "input": format!("Take the given json detailing tech articles and only keep the entries that are related to AI and ML. ONLY OUTPUT PURE JSON, NOTHING ELSE, NOT EVEN MARKDOWN BACKTICK INDICATORS: {}", &json_content)
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

    let mut extracted_json_response =
        serde_json::to_string(&filtered_json_response["output"][0]["content"][0]["text"])?;

    extracted_json_response = extracted_json_response
        .replace("\n", "")
        .replace("\\n", "")
        .replace("\\", "");

    let mut chars = extracted_json_response.chars();
    chars.next();
    chars.next_back();
    let final_json_response = chars.as_str();

    let final_extracted_json: JsonResponse = serde_json::from_str(final_json_response).unwrap();

    Ok(final_extracted_json.response)
}
