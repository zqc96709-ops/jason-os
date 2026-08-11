use rusqlite::{params, Connection, OptionalExtension};
use serde_json::{json, Value};
use std::collections::hash_map::DefaultHasher;
use std::hash::{Hash, Hasher};

pub fn migrate(connection: &Connection, applied_at: &str) -> Result<(), String> {
    connection.execute_batch(
        "CREATE TABLE IF NOT EXISTS external_items (
           id TEXT PRIMARY KEY,
           platform TEXT NOT NULL,
           external_id TEXT,
           content_type TEXT NOT NULL,
           title TEXT NOT NULL DEFAULT '',
           content_excerpt TEXT NOT NULL DEFAULT '',
           author TEXT NOT NULL DEFAULT '',
           author_id TEXT NOT NULL DEFAULT '',
           canonical_url TEXT NOT NULL,
           cover_url TEXT NOT NULL DEFAULT '',
           published_at TEXT,
           captured_at TEXT NOT NULL,
           expires_at TEXT,
           provider TEXT NOT NULL,
           provider_item_id TEXT,
           content_hash TEXT NOT NULL,
           raw_payload_path TEXT,
           UNIQUE(platform, external_id),
           UNIQUE(canonical_url)
         );
         CREATE INDEX IF NOT EXISTS idx_external_items_platform_published ON external_items(platform, published_at DESC);
         CREATE INDEX IF NOT EXISTS idx_external_items_expires ON external_items(expires_at);
         CREATE INDEX IF NOT EXISTS idx_external_items_hash ON external_items(content_hash);
         CREATE TABLE IF NOT EXISTS external_observations (
           id TEXT PRIMARY KEY,
           item_id TEXT NOT NULL,
           observed_at TEXT NOT NULL,
           views INTEGER NOT NULL DEFAULT 0,
           likes INTEGER NOT NULL DEFAULT 0,
           comments INTEGER NOT NULL DEFAULT 0,
           shares INTEGER NOT NULL DEFAULT 0,
           saves INTEGER NOT NULL DEFAULT 0,
           followers INTEGER NOT NULL DEFAULT 0,
           provider TEXT NOT NULL,
           UNIQUE(item_id, observed_at),
           FOREIGN KEY(item_id) REFERENCES external_items(id) ON DELETE CASCADE
         );
         CREATE INDEX IF NOT EXISTS idx_external_observations_item_time ON external_observations(item_id, observed_at DESC);
         CREATE TABLE IF NOT EXISTS external_provider_calls (
           id TEXT PRIMARY KEY,
           provider TEXT NOT NULL,
           endpoint TEXT NOT NULL,
           source_id TEXT,
           called_at TEXT NOT NULL,
           success INTEGER NOT NULL,
           status_code INTEGER,
           estimated_cost_micros INTEGER NOT NULL DEFAULT 0,
           items_returned INTEGER NOT NULL DEFAULT 0,
           error_code TEXT,
           error_message TEXT
         );
         CREATE INDEX IF NOT EXISTS idx_external_provider_calls_time ON external_provider_calls(called_at DESC);
         CREATE INDEX IF NOT EXISTS idx_external_provider_calls_provider ON external_provider_calls(provider, called_at DESC);"
    ).map_err(|error| error.to_string())?;
    connection
        .execute(
            "INSERT OR IGNORE INTO schema_migrations(version, applied_at) VALUES (7, ?1)",
            params![applied_at],
        )
        .map_err(|error| error.to_string())?;
    Ok(())
}

fn value_text(value: &Value, key: &str) -> String {
    value
        .get(key)
        .and_then(Value::as_str)
        .unwrap_or("")
        .trim()
        .to_string()
}

pub fn content_hash(canonical: &Value) -> String {
    let mut hasher = DefaultHasher::new();
    value_text(canonical, "platformCode").hash(&mut hasher);
    value_text(canonical, "title").hash(&mut hasher);
    value_text(canonical, "content").hash(&mut hasher);
    format!("{:016x}", hasher.finish())
}

pub fn upsert_item(
    connection: &Connection,
    id: &str,
    canonical: &Value,
    captured_at: &str,
    expires_at: &str,
    raw_payload_path: &str,
) -> Result<String, String> {
    let platform = value_text(canonical, "platformCode");
    let external_id = value_text(canonical, "externalId");
    let canonical_url = value_text(canonical, "canonicalUrl");
    let existing: Option<String> = if !external_id.is_empty() {
        connection
            .query_row(
                "SELECT id FROM external_items WHERE platform=?1 AND external_id=?2 LIMIT 1",
                params![platform, external_id],
                |row| row.get(0),
            )
            .optional()
            .map_err(|error| error.to_string())?
    } else {
        None
    };
    let existing = match existing {
        Some(value) => Some(value),
        None if !canonical_url.is_empty() => connection
            .query_row(
                "SELECT id FROM external_items WHERE canonical_url=?1 LIMIT 1",
                params![canonical_url],
                |row| row.get(0),
            )
            .optional()
            .map_err(|error| error.to_string())?,
        None => None,
    };
    let item_id = existing.unwrap_or_else(|| id.to_string());
    let metrics = canonical
        .get("metrics")
        .cloned()
        .unwrap_or_else(|| json!({}));
    connection.execute(
        "INSERT INTO external_items(id,platform,external_id,content_type,title,content_excerpt,author,author_id,canonical_url,cover_url,published_at,captured_at,expires_at,provider,provider_item_id,content_hash,raw_payload_path)
         VALUES(?1,?2,NULLIF(?3,''),?4,?5,?6,?7,?8,?9,?10,NULLIF(?11,''),?12,?13,?14,NULLIF(?15,''),?16,?17)
         ON CONFLICT(id) DO UPDATE SET title=excluded.title,content_excerpt=excluded.content_excerpt,author=excluded.author,author_id=excluded.author_id,cover_url=excluded.cover_url,published_at=excluded.published_at,captured_at=excluded.captured_at,expires_at=excluded.expires_at,provider=excluded.provider,content_hash=excluded.content_hash,raw_payload_path=excluded.raw_payload_path",
        params![
            item_id,
            value_text(canonical, "platformCode"),
            value_text(canonical, "externalId"),
            value_text(canonical, "contentType"),
            value_text(canonical, "title"),
            value_text(canonical, "content"),
            value_text(canonical, "author"),
            value_text(canonical, "authorId"),
            value_text(canonical, "canonicalUrl"),
            value_text(canonical, "coverUrl"),
            value_text(canonical, "publishedAt"),
            captured_at,
            expires_at,
            value_text(canonical, "provider"),
            value_text(canonical, "externalId"),
            content_hash(canonical),
            raw_payload_path,
        ],
    ).map_err(|error| error.to_string())?;
    connection.execute(
        "INSERT OR IGNORE INTO external_observations(id,item_id,observed_at,views,likes,comments,shares,saves,followers,provider) VALUES(?1,?2,?3,?4,?5,?6,?7,?8,?9,?10)",
        params![
            format!("observation-{item_id}-{captured_at}"), item_id, captured_at,
            metrics["views"].as_i64().unwrap_or(0), metrics["likes"].as_i64().unwrap_or(0),
            metrics["comments"].as_i64().unwrap_or(0), metrics["shares"].as_i64().unwrap_or(0),
            metrics["saves"].as_i64().unwrap_or(0), metrics["followers"].as_i64().unwrap_or(0),
            value_text(canonical, "provider")
        ],
    ).map_err(|error| error.to_string())?;
    Ok(item_id)
}

pub fn record_provider_call(
    connection: &Connection,
    id: &str,
    provider: &str,
    endpoint: &str,
    source_id: Option<&str>,
    called_at: &str,
    success: bool,
    status_code: Option<u16>,
    items_returned: i64,
    error_message: Option<&str>,
) -> Result<(), String> {
    connection.execute(
        "INSERT INTO external_provider_calls(id,provider,endpoint,source_id,called_at,success,status_code,estimated_cost_micros,items_returned,error_message) VALUES(?1,?2,?3,?4,?5,?6,?7,0,?8,?9)",
        params![id, provider, endpoint, source_id, called_at, success as i64, status_code, items_returned, error_message],
    ).map_err(|error| error.to_string())?;
    Ok(())
}

pub fn list_items(connection: &Connection, limit: i64) -> Result<Vec<Value>, String> {
    let mut statement = connection.prepare(
        "SELECT i.id,i.platform,i.external_id,i.content_type,i.title,i.content_excerpt,i.author,i.canonical_url,i.cover_url,i.published_at,i.captured_at,i.expires_at,i.provider,
                COALESCE(o.views,0),COALESCE(o.likes,0),COALESCE(o.comments,0),COALESCE(o.shares,0),COALESCE(o.saves,0)
         FROM external_items i LEFT JOIN external_observations o ON o.id=(SELECT id FROM external_observations WHERE item_id=i.id ORDER BY observed_at DESC LIMIT 1)
         ORDER BY i.captured_at DESC LIMIT ?1"
    ).map_err(|error| error.to_string())?;
    let rows = statement.query_map(params![limit.clamp(1, 200)], |row| Ok(json!({
        "id": row.get::<_,String>(0)?, "platform": row.get::<_,String>(1)?, "externalId": row.get::<_,Option<String>>(2)?,
        "contentType": row.get::<_,String>(3)?, "title": row.get::<_,String>(4)?, "content": row.get::<_,String>(5)?,
        "author": row.get::<_,String>(6)?, "canonicalUrl": row.get::<_,String>(7)?, "coverUrl": row.get::<_,String>(8)?,
        "publishedAt": row.get::<_,Option<String>>(9)?, "capturedAt": row.get::<_,String>(10)?, "expiresAt": row.get::<_,Option<String>>(11)?,
        "provider": row.get::<_,String>(12)?, "metrics": {"views":row.get::<_,i64>(13)?,"likes":row.get::<_,i64>(14)?,"comments":row.get::<_,i64>(15)?,"shares":row.get::<_,i64>(16)?,"saves":row.get::<_,i64>(17)?}
    }))).map_err(|error| error.to_string())?;
    rows.collect::<Result<Vec<_>, _>>()
        .map_err(|error| error.to_string())
}

pub fn cleanup_expired(connection: &Connection, now: &str) -> Result<i64, String> {
    connection
        .execute(
            "DELETE FROM external_items WHERE expires_at IS NOT NULL AND CAST(expires_at AS INTEGER) < CAST(?1 AS INTEGER)",
            params![now],
        )
        .map(|count| count as i64)
        .map_err(|error| error.to_string())
}

#[cfg(test)]
mod tests {
    use super::*;

    fn db() -> Connection {
        let connection = Connection::open_in_memory().unwrap();
        connection.execute_batch("PRAGMA foreign_keys=ON; CREATE TABLE schema_migrations(version INTEGER PRIMARY KEY,applied_at TEXT NOT NULL);").unwrap();
        migrate(&connection, "1").unwrap();
        connection
    }

    #[test]
    fn migration_v7_creates_required_tables_and_indexes() {
        let connection = db();
        for name in [
            "external_items",
            "external_observations",
            "external_provider_calls",
            "idx_external_items_platform_published",
            "idx_external_items_expires",
            "idx_external_items_hash",
            "idx_external_observations_item_time",
            "idx_external_provider_calls_time",
            "idx_external_provider_calls_provider",
        ] {
            let count: i64 = connection
                .query_row(
                    "SELECT COUNT(*) FROM sqlite_master WHERE name=?1",
                    params![name],
                    |row| row.get(0),
                )
                .unwrap();
            assert_eq!(count, 1, "missing migration object: {name}");
        }
        let version: i64 = connection
            .query_row(
                "SELECT COUNT(*) FROM schema_migrations WHERE version=7",
                [],
                |row| row.get(0),
            )
            .unwrap();
        assert_eq!(version, 1);
    }

    #[test]
    fn stores_items_without_duplicate_urls() {
        let connection = db();
        let canonical = json!({"platformCode":"douyin","externalId":"123","contentType":"VIDEO_POST","title":"Title","content":"Text","canonicalUrl":"https://douyin.com/video/123","provider":"redfox","metrics":{"views":10}});
        let first =
            upsert_item(&connection, "item-1", &canonical, "100", "200", "/tmp/raw").unwrap();
        let second =
            upsert_item(&connection, "item-2", &canonical, "110", "210", "/tmp/raw2").unwrap();
        assert_eq!(first, second);
        let count: i64 = connection
            .query_row("SELECT COUNT(*) FROM external_items", [], |row| row.get(0))
            .unwrap();
        assert_eq!(count, 1);
    }

    #[test]
    fn removes_only_expired_cache_items() {
        let connection = db();
        let a = json!({"platformCode":"douyin","externalId":"a","contentType":"VIDEO_POST","title":"A","canonicalUrl":"https://a","provider":"redfox"});
        let b = json!({"platformCode":"douyin","externalId":"b","contentType":"VIDEO_POST","title":"B","canonicalUrl":"https://b","provider":"redfox"});
        upsert_item(&connection, "a", &a, "1", "5", "").unwrap();
        upsert_item(&connection, "b", &b, "1", "50", "").unwrap();
        assert_eq!(cleanup_expired(&connection, "10").unwrap(), 1);
    }
}
