use reqwest::blocking::Client;
use serde_json::{json, Value};
use std::time::Duration;

const BASE: &str = "https://api.tikhub.io";

pub struct TikHubCapture {
    pub canonical: Value,
    pub raw: Value,
    pub endpoint: String,
    pub status_code: u16,
}

fn client() -> Result<Client, String> {
    Client::builder()
        .timeout(Duration::from_secs(60))
        .build()
        .map_err(|e| format!("TikHub HTTP 客户端初始化失败：{e}"))
}
fn err(status: reqwest::StatusCode, body: &str) -> String {
    format!(
        "TikHub 返回 HTTP {}：{}",
        status.as_u16(),
        body.chars().take(500).collect::<String>()
    )
}
fn text(value: &Value) -> String {
    match value {
        Value::String(s) => s.trim().to_string(),
        Value::Number(n) => n.to_string(),
        _ => String::new(),
    }
}
fn find(value: &Value, keys: &[&str]) -> String {
    match value {
        Value::Object(o) => keys
            .iter()
            .find_map(|k| o.get(*k).map(text).filter(|v| !v.is_empty()))
            .or_else(|| {
                o.values().find_map(|v| {
                    let x = find(v, keys);
                    (!x.is_empty()).then_some(x)
                })
            })
            .unwrap_or_default(),
        Value::Array(a) => a
            .iter()
            .find_map(|v| {
                let x = find(v, keys);
                (!x.is_empty()).then_some(x)
            })
            .unwrap_or_default(),
        _ => String::new(),
    }
}
fn id_between(url: &str, marker: &str) -> Option<String> {
    url.split(marker)
        .nth(1)
        .map(|s| {
            s.split(['?', '&', '/', '#'])
                .next()
                .unwrap_or_default()
                .to_string()
        })
        .filter(|s| !s.is_empty())
}

pub fn test_token(token: &str) -> Result<Value, String> {
    let response = client()?
        .get(format!("{BASE}/api/v1/tikhub/user/get_user_info"))
        .bearer_auth(token)
        .header("Accept", "application/json")
        .send()
        .map_err(|e| format!("TikHub 连通性请求失败：{e}"))?;
    let status = response.status();
    let body = response.text().map_err(|e| e.to_string())?;
    if !status.is_success() {
        return Err(err(status, &body));
    }
    let value: Value =
        serde_json::from_str(&body).map_err(|e| format!("TikHub 响应不是有效 JSON：{e}"))?;
    Ok(json!({"username":find(&value,&["username","email"]),"status":"ok"}))
}

pub fn capture(token: &str, url: &str) -> Result<TikHubCapture, String> {
    let lower = url.to_lowercase();
    let (path, key, value) = if lower.contains("douyin.com") {
        (
            "/api/v1/douyin/web/fetch_one_video_by_share_url",
            "share_url",
            url.to_string(),
        )
    } else if lower.contains("tiktok.com") {
        let id = id_between(url, "/video/").ok_or("无法从 TikTok 链接提取视频 ID".to_string())?;
        ("/api/v1/tiktok/web/fetch_post_detail", "itemId", id)
    } else if lower.contains("x.com") || lower.contains("twitter.com") {
        let id = id_between(url, "/status/").ok_or("无法从 X 链接提取推文 ID".to_string())?;
        ("/api/v1/twitter/web/fetch_tweet_detail", "tweet_id", id)
    } else if lower.contains("xiaohongshu.com") || lower.contains("xhslink.com") {
        let id = id_between(url, "/explore/").ok_or("无法从小红书链接提取笔记 ID".to_string())?;
        (
            "/api/v1/xiaohongshu/web_v3/fetch_note_detail",
            "note_id",
            id,
        )
    } else {
        return Err("TikHub 当前未配置该平台的单条链接接口".into());
    };
    let endpoint = format!("{BASE}{path}");
    let mut request_url = reqwest::Url::parse(&endpoint).map_err(|e| e.to_string())?;
    request_url.query_pairs_mut().append_pair(key, &value);
    let response = client()?
        .get(request_url)
        .bearer_auth(token)
        .header("Accept", "application/json")
        .send()
        .map_err(|e| format!("TikHub 采集请求失败：{e}"))?;
    let status = response.status();
    let code = status.as_u16();
    let body = response.text().map_err(|e| e.to_string())?;
    if !status.is_success() {
        return Err(err(status, &body));
    }
    let raw: Value =
        serde_json::from_str(&body).map_err(|e| format!("TikHub 响应不是有效 JSON：{e}"))?;
    let title = find(&raw, &["title", "desc", "description", "nickname"]);
    let content = find(&raw, &["content", "desc", "description", "text"]);
    Ok(TikHubCapture {
        canonical: json!({"title":if title.is_empty(){url}else{&title},"description":content.chars().take(2000).collect::<String>(),"content":content,"webpage_url":url,"canonicalUrl":url,"contentType":"SOCIAL_POST","provider":"tikhub","providerEndpoint":path,"metrics":{}}),
        raw,
        endpoint,
        status_code: code,
    })
}
