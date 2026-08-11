use reqwest::blocking::Client;
use serde_json::{json, Value};
use std::time::Duration;

pub struct RedfoxCapture {
    pub canonical: Value,
    pub raw: Value,
    pub endpoint: String,
    pub status_code: u16,
}

fn text(value: &Value) -> Option<String> {
    match value {
        Value::String(value) if !value.trim().is_empty() => Some(value.trim().to_string()),
        Value::Number(value) => Some(value.to_string()),
        _ => None,
    }
}

fn find_value(value: &Value, keys: &[&str]) -> Option<Value> {
    match value {
        Value::Object(object) => {
            for key in keys {
                if let Some(found) = object.get(*key) {
                    if !found.is_null() {
                        return Some(found.clone());
                    }
                }
            }
            object.values().find_map(|child| find_value(child, keys))
        }
        Value::Array(values) => values.iter().find_map(|child| find_value(child, keys)),
        _ => None,
    }
}

fn find_text(value: &Value, keys: &[&str]) -> String {
    find_value(value, keys)
        .as_ref()
        .and_then(text)
        .unwrap_or_default()
}

fn find_i64(value: &Value, keys: &[&str]) -> i64 {
    find_value(value, keys)
        .and_then(|value| match value {
            Value::Number(value) => value.as_i64(),
            Value::String(value) => value.replace(',', "").parse::<i64>().ok(),
            _ => None,
        })
        .unwrap_or(0)
}

fn extract_between(url: &str, markers: &[&str]) -> String {
    markers
        .iter()
        .find_map(|marker| {
            let start = url.find(marker)? + marker.len();
            let tail = &url[start..];
            let value = tail.split(['?', '&', '/', '#']).next().unwrap_or("").trim();
            (!value.is_empty()).then(|| value.to_string())
        })
        .unwrap_or_default()
}

pub fn platform_for_url(url: &str) -> Option<&'static str> {
    let lower = url.to_lowercase();
    if lower.contains("mp.weixin.qq.com") {
        Some("wechat")
    } else if lower.contains("douyin.com") {
        Some("douyin")
    } else if lower.contains("xiaohongshu.com") || lower.contains("xhslink.com") {
        Some("xiaohongshu")
    } else {
        None
    }
}

pub fn request_for_url(url: &str) -> Result<(&'static str, Value), String> {
    match platform_for_url(url) {
        Some("wechat") => Ok((
            "https://redfox.work/story/api/gzhData/queryArticleDetail",
            json!({"workUrl": url}),
        )),
        Some("douyin") => {
            let work_id = extract_between(url, &["/video/", "modal_id="]);
            if work_id.is_empty() {
                return Err("无法从抖音链接提取作品 ID，请使用作品完整链接".into());
            }
            Ok((
                "https://redfox.work/story/api/douyinData/queryWorkDetail",
                json!({"workId": work_id}),
            ))
        }
        Some("xiaohongshu") => {
            let work_id = extract_between(url, &["/explore/", "/discovery/item/"]);
            if work_id.is_empty() {
                return Err("无法从小红书链接提取笔记 ID，请使用笔记完整链接".into());
            }
            Ok((
                "https://redfox.work/story/api/xhsUser/queryWorkDetail",
                json!({"workId": work_id}),
            ))
        }
        _ => Err("RedFox 当前未配置该平台的单条内容接口".into()),
    }
}

pub fn normalize_response(url: &str, endpoint: &str, raw: &Value) -> Value {
    let platform = platform_for_url(url).unwrap_or("web");
    let platform_label = match platform {
        "wechat" => "微信公众号",
        "douyin" => "抖音",
        "xiaohongshu" => "小红书",
        _ => "网页",
    };
    let content_type = if platform == "wechat" {
        "ARTICLE"
    } else {
        "VIDEO_POST"
    };
    let title = find_text(raw, &["title", "workTitle", "noteTitle", "shareTitle"]);
    let content = find_text(raw, &["content", "desc", "description", "noteText", "text"]);
    let author = find_text(
        raw,
        &["author", "authorName", "nickname", "nickName", "userName"],
    );
    let author_id = find_text(raw, &["authorId", "userId", "secUid", "uid"]);
    let external_id = find_text(raw, &["workId", "noteId", "awemeId", "articleId", "id"]);
    let published_at = find_text(
        raw,
        &["publishedAt", "publishTime", "createTime", "uploadTime"],
    );
    let canonical_url = {
        let value = find_text(raw, &["workUrl", "url", "shareUrl", "webpageUrl"]);
        if value.starts_with("http") {
            value
        } else {
            url.to_string()
        }
    };
    let cover_url = find_text(raw, &["coverUrl", "cover", "thumbnail", "imageUrl"]);
    json!({
        "platform": platform_label,
        "platformCode": platform,
        "externalId": external_id,
        "contentType": content_type,
        "title": if title.is_empty() { canonical_url.clone() } else { title },
        "content": content,
        "author": author,
        "authorId": author_id,
        "publishedAt": published_at,
        "canonicalUrl": canonical_url,
        "coverUrl": cover_url,
        "mediaUrls": [],
        "metrics": {
            "views": find_i64(raw, &["viewCount", "playCount", "views", "readCount"]),
            "likes": find_i64(raw, &["likeCount", "diggCount", "likes"]),
            "comments": find_i64(raw, &["commentCount", "comments"]),
            "shares": find_i64(raw, &["shareCount", "repostCount", "shares"]),
            "saves": find_i64(raw, &["collectCount", "favoriteCount", "saves"]),
            "followers": find_i64(raw, &["fansCount", "followerCount", "followers"])
        },
        "provider": "redfox",
        "providerEndpoint": endpoint
    })
}

pub fn capture(api_key: &str, url: &str) -> Result<RedfoxCapture, String> {
    let (endpoint, body) = request_for_url(url)?;
    let client = Client::builder()
        .timeout(Duration::from_secs(25))
        .user_agent("JasonOS/3.1 ExternalIntelligence")
        .build()
        .map_err(|error| error.to_string())?;
    let response = client
        .post(endpoint)
        .header("REDFOX_API_KEY", api_key)
        .json(&body)
        .send()
        .map_err(|error| format!("RedFox 请求失败：{error}"))?;
    let status_code = response.status().as_u16();
    let raw: Value = response
        .json()
        .map_err(|error| format!("RedFox 返回内容无法解析：{error}"))?;
    if !(200..300).contains(&status_code) {
        return Err(format!("RedFox HTTP {status_code}"));
    }
    let code = raw.get("code").and_then(Value::as_i64).unwrap_or(200);
    if ![0, 200].contains(&code) {
        let message = find_text(&raw, &["message", "msg", "error"]);
        return Err(format!(
            "RedFox {code}：{}",
            if message.is_empty() {
                "调用失败"
            } else {
                &message
            }
        ));
    }
    Ok(RedfoxCapture {
        canonical: normalize_response(url, endpoint, &raw),
        raw,
        endpoint: endpoint.to_string(),
        status_code,
    })
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn routes_supported_platform_urls() {
        assert!(request_for_url("https://mp.weixin.qq.com/s/abc")
            .unwrap()
            .0
            .contains("gzhData"));
        assert_eq!(
            request_for_url("https://www.douyin.com/video/123456")
                .unwrap()
                .1["workId"],
            "123456"
        );
        assert_eq!(
            request_for_url("https://www.xiaohongshu.com/explore/abc123")
                .unwrap()
                .1["workId"],
            "abc123"
        );
        assert!(request_for_url("https://reddit.com/r/test").is_err());
    }

    #[test]
    fn normalizes_provider_specific_fields() {
        let raw = json!({"code":200,"data":{"workId":"123","desc":"低噪音宠物饮水器","nickname":"Jason","playCount":"1200","diggCount":88,"commentCount":12,"shareCount":4}});
        let canonical = normalize_response("https://www.douyin.com/video/123", "endpoint", &raw);
        assert_eq!(canonical["externalId"], "123");
        assert_eq!(canonical["content"], "低噪音宠物饮水器");
        assert_eq!(canonical["metrics"]["views"], 1200);
        assert_eq!(canonical["metrics"]["likes"], 88);
    }
}
