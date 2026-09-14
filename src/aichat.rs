use serde_json::{ Value };
use reqwest::Client;
use anyhow::{ Result, anyhow, Context };

pub fn extract_chat_result(chat_result: &Value) -> (String, Value, String, u64) {
    let content = chat_result["choices"][0]["message"]["content"]
        .as_str()
        .unwrap_or_default();
    let reasoning = chat_result["choices"][0]["message"]["reasoning_content"]
        .as_str()
        .unwrap_or_default();
    // 直接取出 Value，如果是 None 则创建新的空数组
    let tool_calls = chat_result["choices"][0]["message"]
        .get("tool_calls")
        .map(|v| v.clone())
        .unwrap_or_else(|| Value::Array(Vec::new()));
    let total_tokens = chat_result["usage"]["total_tokens"].as_u64().unwrap_or(0);
    (
        content.to_string(),
        tool_calls,
        reasoning.to_string(),
        total_tokens,
    )
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
