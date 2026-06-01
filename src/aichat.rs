use serde_json::{ Value };
use reqwest::Client;
use anyhow::{ Result, anyhow, Context };

pub fn extract_chat_result(chat_result: &Value) -> String {
    let content = chat_result["choices"][0]["message"]["content"]
        .as_str()
        .unwrap_or_default();
    content.to_string()
}

pub async fn chat(base_url: &str, api_key: &str, payload: Value) -> Result<Value> {
    let client = Client::new();
    let url = format!("{}/chat/completions", base_url);
    let response = client
        .post(url)
        .header("Authorization", format!("Bearer {}", api_key))
        .header("Content-Type", "application/json")
        .json(&payload)
        .send()
        .await?;

    if !response.status().is_success() {
        let error_json: Value = response.json().await.context("请求失败")?;
        let error_message = error_json["error"]["message"]
            .as_str()
            .unwrap_or("请求失败");
        return Err(anyhow!("API 请求失败: {}", error_message));
    }

    let chat_response: Value = response.json().await?;
    Ok(chat_response)
}
