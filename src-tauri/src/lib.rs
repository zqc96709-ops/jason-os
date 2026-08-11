mod external_intelligence;
mod redfox;

use regex::Regex;
use rusqlite::{params, Connection, OptionalExtension};
use serde_json::{json, Map, Value};
#[cfg(unix)]
use std::os::unix::fs::PermissionsExt;
use std::{
    fs,
    hash::{Hash, Hasher},
    path::{Path, PathBuf},
    process::Command,
    sync::atomic::{AtomicU64, Ordering},
    time::{Duration, Instant, SystemTime, UNIX_EPOCH},
};
use tauri::{AppHandle, Manager};

static ID_SEQUENCE: AtomicU64 = AtomicU64::new(0);

const ENTITIES: &[&str] = &[
    "goals",
    "keyResults",
    "projects",
    "tasks",
    "hypotheses",
    "experiments",
    "timeLogs",
    "results",
    "reviews",
    "knowledge",
    "insights",
    "principles",
    "mentalModels",
    "mentalModelUsages",
    "decisions",
    "inbox",
    "events",
    "people",
    "dataRecords",
    "attachments",
    "timelineEvents",
    "agentRuns",
    "agentActions",
    "externalSources",
    "signals",
    "opportunities",
    "intelligenceBriefs",
];

fn now() -> String {
    let ms = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .unwrap_or_default()
        .as_millis();
    format!("{ms}")
}

fn new_id(entity: &str) -> String {
    format!(
        "{}-{}-{}",
        entity,
        now(),
        ID_SEQUENCE.fetch_add(1, Ordering::Relaxed)
    )
}

fn is_entity(entity: &str) -> bool {
    ENTITIES.contains(&entity)
}

fn data_dir(app: &AppHandle) -> Result<PathBuf, String> {
    let dir = app.path().app_data_dir().map_err(|e| e.to_string())?;
    fs::create_dir_all(dir.join("attachments")).map_err(|e| e.to_string())?;
    fs::create_dir_all(dir.join("exports")).map_err(|e| e.to_string())?;
    fs::create_dir_all(dir.join("backups")).map_err(|e| e.to_string())?;
    fs::create_dir_all(dir.join("external-intelligence/raw")).map_err(|e| e.to_string())?;
    Ok(dir)
}

const TIMELINE_SOURCE_ENTITIES: &[&str] = &[
    "goals",
    "projects",
    "tasks",
    "timeLogs",
    "events",
    "results",
    "reviews",
    "insights",
    "decisions",
    "signals",
    "opportunities",
    "intelligenceBriefs",
    "timelineEvents",
];

fn timeline_text(data: &Value, key: &str) -> Option<String> {
    data.get(key)
        .and_then(Value::as_str)
        .map(str::trim)
        .filter(|value| !value.is_empty())
        .map(str::to_string)
}

fn first_timeline_value(data: &Value, keys: &[&str]) -> Option<String> {
    keys.iter().find_map(|key| timeline_text(data, key))
}

fn timeline_semantics(
    entity: &str,
    data: &Value,
    created_at: &str,
    updated_at: &str,
) -> (String, &'static str) {
    match entity {
        "timeLogs" => (
            first_timeline_value(data, &["startAt"]).unwrap_or_else(|| created_at.into()),
            "actual",
        ),
        "events" => (
            first_timeline_value(data, &["startAt"]).unwrap_or_else(|| created_at.into()),
            "planned",
        ),
        "decisions" => (
            first_timeline_value(data, &["date", "decisionDate"])
                .unwrap_or_else(|| created_at.into()),
            "recorded",
        ),
        "results" => (
            first_timeline_value(data, &["date", "completedAt"])
                .unwrap_or_else(|| created_at.into()),
            "actual",
        ),
        "signals" => (
            first_timeline_value(data, &["detectedAt"]).unwrap_or_else(|| created_at.into()),
            "recorded",
        ),
        "intelligenceBriefs" => (
            first_timeline_value(data, &["generatedAt"]).unwrap_or_else(|| created_at.into()),
            "recorded",
        ),
        "tasks" if data.get("status").and_then(Value::as_str) == Some("completed") => (
            first_timeline_value(data, &["completedAt"]).unwrap_or_else(|| updated_at.into()),
            "actual",
        ),
        "tasks" => first_timeline_value(data, &["dueAt", "dueDate"])
            .map(|value| (value, "planned"))
            .unwrap_or_else(|| (created_at.into(), "recorded")),
        "timelineEvents" => (
            first_timeline_value(data, &["occurredAt"]).unwrap_or_else(|| created_at.into()),
            "actual",
        ),
        _ => (created_at.into(), "recorded"),
    }
}

fn timeline_time_zone(value: &str) -> &'static str {
    if value.chars().all(|character| character.is_ascii_digit()) || value.ends_with('Z') {
        "UTC"
    } else if value.rfind(['+', '-']).is_some_and(|index| index > 9) {
        "offset"
    } else {
        "local"
    }
}

fn timeline_precision(value: &str) -> &'static str {
    if value.len() == 10 && value.chars().nth(4) == Some('-') && value.chars().nth(7) == Some('-') {
        "date"
    } else {
        "datetime"
    }
}

fn timeline_importance(entity: &str, data: &Value) -> &'static str {
    if data.get("timelineImportance").and_then(Value::as_str) == Some("key") {
        return "key";
    }
    if [
        "decisions",
        "results",
        "reviews",
        "insights",
        "signals",
        "opportunities",
        "intelligenceBriefs",
    ]
    .contains(&entity)
    {
        return "key";
    }
    if entity == "projects"
        && (["blocked", "completed"]
            .contains(&data.get("status").and_then(Value::as_str).unwrap_or(""))
            || ["at_risk", "blocked"]
                .contains(&data.get("health").and_then(Value::as_str).unwrap_or("")))
    {
        return "key";
    }
    if entity == "goals"
        && ["completed", "paused"]
            .contains(&data.get("status").and_then(Value::as_str).unwrap_or(""))
    {
        return "key";
    }
    if entity == "tasks"
        && data.get("status").and_then(Value::as_str) == Some("completed")
        && (data.get("priority").and_then(Value::as_str) == Some("high")
            || data.get("importance").and_then(Value::as_str) == Some("important"))
    {
        return "key";
    }
    "normal"
}

fn timeline_evidence_level(entity: &str, data: &Value) -> &'static str {
    match data.get("evidenceLevel").and_then(Value::as_str) {
        Some("REALITY") => return "REALITY",
        Some("USER_CONFIRMED") => return "USER_CONFIRMED",
        Some("AI_CONFIRMED") => return "AI_CONFIRMED",
        Some("AI_SUGGESTION") => return "AI_SUGGESTION",
        _ => {}
    }
    if ["timeLogs", "results", "timelineEvents"].contains(&entity) {
        "REALITY"
    } else if data.get("agentActionId").and_then(Value::as_str).is_some() {
        "AI_CONFIRMED"
    } else {
        "USER_CONFIRMED"
    }
}

fn apply_timeline_metadata(
    entity: &str,
    data: &Value,
    created_at: &str,
    updated_at: &str,
) -> Value {
    if !TIMELINE_SOURCE_ENTITIES.contains(&entity) {
        return data.clone();
    }
    let mut enriched = data.clone();
    if !enriched.is_object() {
        enriched = json!({});
    }
    let (occurred_at, meaning) = timeline_semantics(entity, &enriched, created_at, updated_at);
    let importance = timeline_importance(entity, &enriched);
    let evidence = timeline_evidence_level(entity, &enriched);
    let object = enriched.as_object_mut().unwrap();
    object.insert("occurredAt".into(), Value::String(occurred_at.clone()));
    object.insert("timeMeaning".into(), Value::String(meaning.into()));
    object.insert(
        "timeZone".into(),
        Value::String(timeline_time_zone(&occurred_at).into()),
    );
    object.insert(
        "timePrecision".into(),
        Value::String(timeline_precision(&occurred_at).into()),
    );
    object.insert(
        "timelineImportance".into(),
        Value::String(importance.into()),
    );
    object.insert("evidenceLevel".into(), Value::String(evidence.into()));
    enriched
}

fn timeline_change_specs(entity: &str, before: &Value, after: &Value) -> Vec<Value> {
    let mut changes = Vec::new();
    let mut add = |field: &str, event_type: &str, title: &str, importance: &str| {
        let old = before.get(field).cloned().unwrap_or(Value::Null);
        let new = after.get(field).cloned().unwrap_or(Value::Null);
        if old != new {
            changes.push(json!({"field":field,"eventType":event_type,"title":title,"beforeValue":old,"afterValue":new,"timelineImportance":importance}));
        }
    };
    match entity {
        "goals" => add("status", "goal_status_changed", "目标状态发生变化", "key"),
        "projects" => {
            add(
                "status",
                "project_status_changed",
                "项目状态发生变化",
                "key",
            );
            add(
                "health",
                "project_health_changed",
                "项目健康度发生变化",
                "key",
            );
            add(
                "blockers",
                "project_blockers_changed",
                "项目阻塞发生变化",
                "key",
            );
        }
        "tasks" => {
            add(
                "status",
                "task_status_changed",
                "任务状态发生变化",
                if after.get("status").and_then(Value::as_str) == Some("completed") {
                    "key"
                } else {
                    "normal"
                },
            );
            add(
                "dueDate",
                "task_due_date_changed",
                "任务截止日期发生变化",
                "normal",
            );
            add(
                "dueAt",
                "task_due_time_changed",
                "任务具体时间发生变化",
                "normal",
            );
        }
        "decisions" => add(
            "status",
            "decision_status_changed",
            "决策状态发生变化",
            "key",
        ),
        "signals" => add(
            "status",
            "signal_status_changed",
            "外部信号状态发生变化",
            "key",
        ),
        "opportunities" => add(
            "status",
            "opportunity_status_changed",
            "机会状态发生变化",
            "key",
        ),
        _ => {}
    }
    changes
}

fn timeline_event_key(source_id: &str, change: &Value) -> String {
    let mut hasher = std::collections::hash_map::DefaultHasher::new();
    source_id.hash(&mut hasher);
    change.to_string().hash(&mut hasher);
    format!("timeline:{:016x}", hasher.finish())
}

fn write_timeline_change_events(
    connection: &Connection,
    entity: &str,
    source_id: &str,
    before: Option<&Value>,
    after: &Value,
) -> Result<(), String> {
    if entity == "timelineEvents" {
        return Ok(());
    }
    let Some(before) = before else {
        return Ok(());
    };
    for change in timeline_change_specs(entity, before, after) {
        let key = timeline_event_key(source_id, &change);
        let exists: bool = connection.query_row("SELECT EXISTS(SELECT 1 FROM records WHERE entity='timelineEvents' AND deleted_at IS NULL AND json_extract(data_json,'$.timelineEventKey')=?1)", params![key], |row| row.get(0)).map_err(|error| error.to_string())?;
        if exists {
            continue;
        }
        let occurred_at = now();
        let mut event = json!({
            "title": change["title"], "eventType": change["eventType"], "occurredAt": occurred_at,
            "timeMeaning": "actual", "timeZone": "UTC", "timePrecision": "datetime",
            "timelineImportance": change["timelineImportance"], "evidenceLevel": "REALITY",
            "sourceEntityType": entity, "sourceEntityId": source_id, "timelineEventKey": key,
            "beforeValue": change["beforeValue"], "afterValue": change["afterValue"]
        });
        if let Some(object) = event.as_object_mut() {
            for field in ["goalId", "projectId", "taskId"] {
                if let Some(value) = after.get(field).and_then(Value::as_str) {
                    object.insert(field.into(), Value::String(value.into()));
                }
            }
            if entity == "goals" {
                object.insert("goalId".into(), Value::String(source_id.into()));
            }
            if entity == "projects" {
                object.insert("projectId".into(), Value::String(source_id.into()));
            }
            if entity == "tasks" {
                object.insert("taskId".into(), Value::String(source_id.into()));
            }
        }
        let id = new_id("timelineEvents");
        let saved = write_record(
            connection,
            "timelineEvents",
            &id,
            &event,
            Some(&occurred_at),
        )?;
        sync_relations(connection, &id, &saved)?;
    }
    Ok(())
}

fn db(app: &AppHandle) -> Result<Connection, String> {
    let connection =
        Connection::open(data_dir(app)?.join("jason-os.sqlite3")).map_err(|e| e.to_string())?;
    connection
        .execute_batch(
            "PRAGMA foreign_keys = ON;
             PRAGMA journal_mode = WAL;
             PRAGMA synchronous = NORMAL;
             CREATE TABLE IF NOT EXISTS schema_migrations (version INTEGER PRIMARY KEY, applied_at TEXT NOT NULL);
             CREATE TABLE IF NOT EXISTS records (
               id TEXT PRIMARY KEY,
               entity TEXT NOT NULL,
               data_json TEXT NOT NULL,
               title TEXT NOT NULL DEFAULT '',
               body TEXT NOT NULL DEFAULT '',
               created_at TEXT NOT NULL,
               updated_at TEXT NOT NULL,
               archived_at TEXT,
               deleted_at TEXT,
               CHECK (entity <> '')
             );
             CREATE INDEX IF NOT EXISTS idx_records_entity_updated ON records(entity, updated_at DESC);
             CREATE VIRTUAL TABLE IF NOT EXISTS records_fts USING fts5(id UNINDEXED, entity UNINDEXED, title, body, tokenize='unicode61');
             CREATE TABLE IF NOT EXISTS settings (key TEXT PRIMARY KEY, value TEXT NOT NULL, updated_at TEXT NOT NULL);
             CREATE TABLE IF NOT EXISTS relations (
               id TEXT PRIMARY KEY,
               from_id TEXT NOT NULL,
               to_id TEXT NOT NULL,
               relation_type TEXT NOT NULL,
               created_at TEXT NOT NULL,
               UNIQUE(from_id, to_id, relation_type),
               FOREIGN KEY(from_id) REFERENCES records(id) ON DELETE CASCADE,
               FOREIGN KEY(to_id) REFERENCES records(id) ON DELETE CASCADE
             );
             CREATE INDEX IF NOT EXISTS idx_relations_from ON relations(from_id, relation_type);
             CREATE INDEX IF NOT EXISTS idx_relations_to ON relations(to_id, relation_type);
             INSERT OR IGNORE INTO schema_migrations(version, applied_at) VALUES (1, strftime('%s','now'));
             INSERT OR IGNORE INTO schema_migrations(version, applied_at) VALUES (2, strftime('%s','now'));
            ",
        )
        .map_err(|e| e.to_string())?;
    ensure_column(&connection, "records", "archived_at", "TEXT")?;
    ensure_column(&connection, "records", "deleted_at", "TEXT")?;
    connection
        .execute_batch(
            "CREATE INDEX IF NOT EXISTS idx_records_active ON records(entity, archived_at, deleted_at, updated_at DESC);
             INSERT OR IGNORE INTO schema_migrations(version, applied_at) VALUES (3, strftime('%s','now'));",
        )
        .map_err(|e| e.to_string())?;
    let relations_backfilled: bool = connection
        .query_row(
            "SELECT EXISTS(SELECT 1 FROM schema_migrations WHERE version=4)",
            [],
            |row| row.get(0),
        )
        .map_err(|e| e.to_string())?;
    if !relations_backfilled {
        let rows = {
            let mut statement = connection
                .prepare("SELECT id, data_json FROM records WHERE deleted_at IS NULL")
                .map_err(|e| e.to_string())?;
            let mapped = statement
                .query_map([], |row| {
                    Ok((row.get::<_, String>(0)?, row.get::<_, String>(1)?))
                })
                .map_err(|e| e.to_string())?;
            mapped
                .collect::<Result<Vec<_>, _>>()
                .map_err(|e| e.to_string())?
        };
        for (id, raw) in rows {
            if let Ok(data) = serde_json::from_str::<Value>(&raw) {
                sync_relations(&connection, &id, &data)?;
            }
        }
        connection
            .execute(
                "INSERT INTO schema_migrations(version, applied_at) VALUES (4, ?1)",
                params![now()],
            )
            .map_err(|e| e.to_string())?;
    }
    let relationships_repaired: bool = connection
        .query_row(
            "SELECT EXISTS(SELECT 1 FROM schema_migrations WHERE version=5)",
            [],
            |row| row.get(0),
        )
        .map_err(|e| e.to_string())?;
    if !relationships_repaired {
        let rows = all_records(&connection)?;
        for record in rows {
            let entity = record["entity"].as_str().unwrap_or("");
            let id = record["id"].as_str().unwrap_or("");
            if entity.is_empty() || id.is_empty() {
                continue;
            }
            let normalized = normalize_record(&connection, entity, &record, false)?;
            let created = record["createdAt"].as_str();
            write_record(&connection, entity, id, &normalized, created)?;
            sync_relations(&connection, id, &normalized)?;
        }
        connection
            .execute(
                "INSERT INTO schema_migrations(version, applied_at) VALUES (5, ?1)",
                params![now()],
            )
            .map_err(|e| e.to_string())?;
    }
    let timeline_backfilled: bool = connection
        .query_row(
            "SELECT EXISTS(SELECT 1 FROM schema_migrations WHERE version=6)",
            [],
            |row| row.get(0),
        )
        .map_err(|error| error.to_string())?;
    if !timeline_backfilled {
        let rows = {
            let mut statement = connection.prepare("SELECT id, entity, data_json, created_at, updated_at FROM records WHERE deleted_at IS NULL").map_err(|error| error.to_string())?;
            let mapped = statement
                .query_map([], |row| {
                    Ok((
                        row.get::<_, String>(0)?,
                        row.get::<_, String>(1)?,
                        row.get::<_, String>(2)?,
                        row.get::<_, String>(3)?,
                        row.get::<_, String>(4)?,
                    ))
                })
                .map_err(|error| error.to_string())?;
            mapped
                .collect::<Result<Vec<_>, _>>()
                .map_err(|error| error.to_string())?
        };
        for (id, entity, raw, created_at, updated_at) in rows {
            if !TIMELINE_SOURCE_ENTITIES.contains(&entity.as_str()) {
                continue;
            }
            let data = serde_json::from_str::<Value>(&raw).unwrap_or_else(|_| json!({}));
            let enriched = apply_timeline_metadata(&entity, &data, &created_at, &updated_at);
            if enriched != data {
                connection
                    .execute(
                        "UPDATE records SET data_json=?2 WHERE id=?1",
                        params![
                            id,
                            serde_json::to_string(&enriched).map_err(|error| error.to_string())?
                        ],
                    )
                    .map_err(|error| error.to_string())?;
            }
        }
        connection
            .execute(
                "INSERT INTO schema_migrations(version, applied_at) VALUES (6, ?1)",
                params![now()],
            )
            .map_err(|error| error.to_string())?;
    }
    external_intelligence::migrate(&connection, &now())?;
    Ok(connection)
}

fn ensure_column(
    connection: &Connection,
    table: &str,
    column: &str,
    definition: &str,
) -> Result<(), String> {
    let mut statement = connection
        .prepare(&format!("PRAGMA table_info({table})"))
        .map_err(|e| e.to_string())?;
    let exists = statement
        .query_map([], |row| row.get::<_, String>(1))
        .map_err(|e| e.to_string())?
        .filter_map(Result::ok)
        .any(|name| name == column);
    if !exists {
        connection
            .execute_batch(&format!(
                "ALTER TABLE {table} ADD COLUMN {column} {definition}"
            ))
            .map_err(|e| e.to_string())?;
    }
    Ok(())
}

fn value_to_record(
    id: String,
    entity: String,
    raw: String,
    created_at: String,
    updated_at: String,
) -> Value {
    let mut data = serde_json::from_str::<Value>(&raw).unwrap_or_else(|_| json!({}));
    if !data.is_object() {
        data = Value::Object(Map::new());
    }
    let object = data.as_object_mut().expect("object just created");
    object.insert("id".into(), Value::String(id));
    object.insert("entity".into(), Value::String(entity));
    object.insert("createdAt".into(), Value::String(created_at));
    object.insert("updatedAt".into(), Value::String(updated_at));
    data
}

fn title_body(data: &Value) -> (String, String) {
    let get = |key: &str| data.get(key).and_then(Value::as_str).unwrap_or("");
    let title = [get("title"), get("name"), get("statement"), get("content")]
        .into_iter()
        .find(|v| !v.trim().is_empty())
        .unwrap_or("")
        .to_string();
    let body = serde_json::to_string(data).unwrap_or_default();
    (title, body)
}

fn write_record(
    connection: &Connection,
    entity: &str,
    id: &str,
    data: &Value,
    created_at: Option<&str>,
) -> Result<Value, String> {
    let timestamp = now();
    let created = created_at.unwrap_or(&timestamp);
    let (title, body) = title_body(data);
    let raw = serde_json::to_string(data).map_err(|e| e.to_string())?;
    connection
        .execute(
            "INSERT INTO records (id, entity, data_json, title, body, created_at, updated_at)
     VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7)
     ON CONFLICT(id) DO UPDATE SET entity=excluded.entity, data_json=excluded.data_json,
       title=excluded.title, body=excluded.body, updated_at=excluded.updated_at",
            params![id, entity, raw, title, body, created, timestamp],
        )
        .map_err(|e| e.to_string())?;
    connection
        .execute("DELETE FROM records_fts WHERE id = ?1", params![id])
        .map_err(|e| e.to_string())?;
    connection
        .execute(
            "INSERT INTO records_fts (id, entity, title, body) VALUES (?1, ?2, ?3, ?4)",
            params![id, entity, title, body],
        )
        .map_err(|e| e.to_string())?;
    Ok(value_to_record(
        id.into(),
        entity.into(),
        serde_json::to_string(data).unwrap_or_default(),
        created.into(),
        timestamp,
    ))
}

fn record_by_id(connection: &Connection, id: &str) -> Result<Option<Value>, String> {
    connection
        .query_row(
            "SELECT id, entity, data_json, created_at, updated_at FROM records WHERE id=?1 AND deleted_at IS NULL",
            params![id],
            |row| {
                Ok(value_to_record(
                    row.get(0)?,
                    row.get(1)?,
                    row.get(2)?,
                    row.get(3)?,
                    row.get(4)?,
                ))
            },
        )
        .optional()
        .map_err(|e| e.to_string())
}

fn string_field(data: &Value, key: &str) -> Option<String> {
    data.get(key)
        .and_then(Value::as_str)
        .map(str::trim)
        .filter(|value| !value.is_empty())
        .map(str::to_string)
}

fn set_optional_id(object: &mut Map<String, Value>, key: &str, value: Option<String>) {
    if let Some(value) = value.filter(|value| !value.is_empty()) {
        object.insert(key.into(), Value::String(value));
    } else {
        object.remove(key);
    }
}

fn expected_entity(key: &str) -> Option<&'static str> {
    match key.trim_end_matches('s') {
        "goalId" => Some("goals"),
        "projectId" => Some("projects"),
        "taskId" | "dependencyId" => Some("tasks"),
        "timeLogId" => Some("timeLogs"),
        "resultId" => Some("results"),
        "reviewId" => Some("reviews"),
        "insightId" => Some("insights"),
        "knowledgeId" => Some("knowledge"),
        "mentalModelId" => Some("mentalModels"),
        "principleId" => Some("principles"),
        "decisionId" => Some("decisions"),
        "hypothesisId" => Some("hypotheses"),
        "experimentId" => Some("experiments"),
        "personId" => Some("people"),
        "sourceId" => Some("externalSources"),
        "signalId" => Some("signals"),
        "opportunityId" => Some("opportunities"),
        "briefId" => Some("intelligenceBriefs"),
        "sourceDecisionId" => Some("decisions"),
        "sourceOpportunityId" => Some("opportunities"),
        "sourceSignalId" => Some("signals"),
        _ => None,
    }
}

fn valid_reference(
    connection: &Connection,
    id: &str,
    expected: &str,
    strict: bool,
) -> Result<Option<Value>, String> {
    let record = record_by_id(connection, id)?;
    if record.as_ref().and_then(|record| record["entity"].as_str()) == Some(expected) {
        return Ok(record);
    }
    if strict {
        Err(format!("关联记录无效：{id} 不是有效的 {expected}"))
    } else {
        Ok(None)
    }
}

fn align_context_from_reference(
    connection: &Connection,
    normalized: &mut Value,
    relation_key: &str,
    expected_entity: &str,
    context_keys: &[&str],
    strict: bool,
) -> Result<(), String> {
    let Some(reference_id) = string_field(normalized, relation_key) else {
        return Ok(());
    };
    let Some(reference) = valid_reference(connection, &reference_id, expected_entity, strict)?
    else {
        normalized.as_object_mut().unwrap().remove(relation_key);
        return Ok(());
    };
    for key in context_keys {
        if let (Some(current), Some(parent)) =
            (string_field(normalized, key), string_field(&reference, key))
        {
            if current != parent {
                if strict {
                    return Err(format!(
                        "因果关联跨越不同上下文：{relation_key} 的 {key} 不一致"
                    ));
                }
                normalized.as_object_mut().unwrap().remove(relation_key);
                return Ok(());
            }
        }
    }
    let inherited = context_keys
        .iter()
        .filter(|key| string_field(normalized, key).is_none())
        .map(|key| (*key, string_field(&reference, key)))
        .collect::<Vec<_>>();
    let object = normalized.as_object_mut().unwrap();
    for (key, value) in inherited {
        set_optional_id(object, key, value);
    }
    Ok(())
}

fn normalize_record(
    connection: &Connection,
    entity: &str,
    data: &Value,
    strict: bool,
) -> Result<Value, String> {
    let mut normalized = data.clone();
    if !normalized.is_object() {
        normalized = Value::Object(Map::new());
    }

    if entity == "projects" {
        if let Some(goal_id) = string_field(&normalized, "goalId") {
            if valid_reference(connection, &goal_id, "goals", strict)?.is_none() {
                normalized.as_object_mut().unwrap().remove("goalId");
            }
        }
    }

    if entity == "tasks" {
        if let Some(project_id) = string_field(&normalized, "projectId") {
            if let Some(project) = valid_reference(connection, &project_id, "projects", strict)? {
                let inherited_goal = string_field(&project, "goalId");
                let object = normalized.as_object_mut().unwrap();
                set_optional_id(object, "projectId", Some(project_id));
                set_optional_id(object, "goalId", inherited_goal);
            } else {
                normalized.as_object_mut().unwrap().remove("projectId");
            }
        }
        if string_field(&normalized, "projectId").is_none() {
            if let Some(goal_id) = string_field(&normalized, "goalId") {
                if valid_reference(connection, &goal_id, "goals", strict)?.is_none() {
                    normalized.as_object_mut().unwrap().remove("goalId");
                }
            }
        }
    }

    if ["timeLogs", "results", "reviews", "decisions"].contains(&entity) {
        if let Some(task_id) = string_field(&normalized, "taskId") {
            if let Some(task) = valid_reference(connection, &task_id, "tasks", strict)? {
                let object = normalized.as_object_mut().unwrap();
                set_optional_id(object, "taskId", Some(task_id));
                set_optional_id(object, "projectId", string_field(&task, "projectId"));
                set_optional_id(object, "goalId", string_field(&task, "goalId"));
            } else {
                normalized.as_object_mut().unwrap().remove("taskId");
            }
        }
        if string_field(&normalized, "taskId").is_none() {
            if let Some(project_id) = string_field(&normalized, "projectId") {
                if let Some(project) = valid_reference(connection, &project_id, "projects", strict)?
                {
                    let object = normalized.as_object_mut().unwrap();
                    set_optional_id(object, "projectId", Some(project_id));
                    set_optional_id(object, "goalId", string_field(&project, "goalId"));
                } else {
                    normalized.as_object_mut().unwrap().remove("projectId");
                }
            }
        }
    }

    if entity == "tasks" {
        align_context_from_reference(
            connection,
            &mut normalized,
            "decisionId",
            "decisions",
            &["projectId", "goalId"],
            strict,
        )?;
    }

    if entity == "reviews" {
        align_context_from_reference(
            connection,
            &mut normalized,
            "resultId",
            "results",
            &["taskId", "projectId", "goalId"],
            strict,
        )?;
    }

    if entity == "insights" {
        if string_field(&normalized, "reviewId").is_some() {
            align_context_from_reference(
                connection,
                &mut normalized,
                "reviewId",
                "reviews",
                &["taskId", "projectId", "goalId"],
                strict,
            )?;
        } else {
            align_context_from_reference(
                connection,
                &mut normalized,
                "resultId",
                "results",
                &["taskId", "projectId", "goalId"],
                strict,
            )?;
        }
    }

    if entity == "timeLogs" {
        let running = normalized["isRunning"].as_bool().unwrap_or(false);
        let object = normalized.as_object_mut().unwrap();
        object.insert(
            "status".into(),
            Value::String(if running { "running" } else { "completed" }.into()),
        );
    }

    let keys = normalized
        .as_object()
        .map(|object| object.keys().cloned().collect::<Vec<_>>())
        .unwrap_or_default();
    for key in keys {
        let Some(expected) = expected_entity(&key) else {
            continue;
        };
        if key.ends_with("Ids") {
            let values = match normalized.get(&key) {
                Some(Value::Array(values)) => values
                    .iter()
                    .filter_map(Value::as_str)
                    .map(str::to_string)
                    .collect::<Vec<_>>(),
                Some(Value::String(values)) => values
                    .split(',')
                    .map(str::trim)
                    .filter(|value| !value.is_empty())
                    .map(str::to_string)
                    .collect(),
                _ => Vec::new(),
            };
            let mut valid = Vec::new();
            for id in values {
                if valid_reference(connection, &id, expected, strict)?.is_some() {
                    valid.push(Value::String(id));
                }
            }
            normalized
                .as_object_mut()
                .unwrap()
                .insert(key, Value::Array(valid));
        } else if let Some(id) = string_field(&normalized, &key) {
            if valid_reference(connection, &id, expected, strict)?.is_none() {
                normalized.as_object_mut().unwrap().remove(&key);
            }
        }
    }
    Ok(normalized)
}

fn cascade_context(
    connection: &Connection,
    parent_entity: &str,
    parent_id: &str,
) -> Result<(), String> {
    let rows = all_records(connection)?;
    for record in rows {
        let matches = (parent_entity == "projects"
            && string_field(&record, "projectId").as_deref() == Some(parent_id))
            || (parent_entity == "tasks"
                && string_field(&record, "taskId").as_deref() == Some(parent_id));
        if !matches {
            continue;
        }
        let Some(entity) = record["entity"].as_str() else {
            continue;
        };
        let Some(id) = record["id"].as_str() else {
            continue;
        };
        let normalized = normalize_record(connection, entity, &record, false)?;
        let created = record["createdAt"].as_str();
        write_record(connection, entity, id, &normalized, created)?;
        sync_relations(connection, id, &normalized)?;
    }
    Ok(())
}

fn relationship_integrity(connection: &Connection) -> Result<Value, String> {
    let records = all_records(connection)?;
    let mut issues = Vec::new();
    for record in records {
        let entity = record["entity"].as_str().unwrap_or("");
        let id = record["id"].as_str().unwrap_or("");
        let normalized = normalize_record(connection, entity, &record, false)?;
        for key in [
            "goalId",
            "projectId",
            "taskId",
            "resultId",
            "reviewId",
            "insightId",
        ] {
            if record.get(key) != normalized.get(key) {
                issues.push(
                    json!({"id": id, "entity": entity, "field": key, "value": record.get(key)}),
                );
            }
        }
    }
    Ok(json!({"ok": issues.is_empty(), "issueCount": issues.len(), "issues": issues}))
}

fn sync_relations(connection: &Connection, source_id: &str, data: &Value) -> Result<(), String> {
    connection
        .execute(
            "DELETE FROM relations WHERE from_id=?1 AND relation_type LIKE 'field:%'",
            params![source_id],
        )
        .map_err(|e| e.to_string())?;
    let Some(object) = data.as_object() else {
        return Ok(());
    };
    for (key, value) in object {
        if key == "id" || key == "entity" || key == "sourceId" {
            continue;
        }
        let targets: Vec<String> = if key.ends_with("Id") {
            value
                .as_str()
                .filter(|v| !v.trim().is_empty())
                .map(|v| vec![v.trim().to_string()])
                .unwrap_or_default()
        } else if key.ends_with("Ids") {
            match value {
                Value::Array(values) => values
                    .iter()
                    .filter_map(Value::as_str)
                    .map(str::trim)
                    .filter(|v| !v.is_empty())
                    .map(str::to_string)
                    .collect(),
                Value::String(values) => values
                    .split(',')
                    .map(str::trim)
                    .filter(|v| !v.is_empty())
                    .map(str::to_string)
                    .collect(),
                _ => Vec::new(),
            }
        } else {
            Vec::new()
        };
        for target in targets {
            let target_exists: bool = connection
                .query_row(
                    "SELECT EXISTS(SELECT 1 FROM records WHERE id=?1 AND deleted_at IS NULL)",
                    params![target],
                    |row| row.get(0),
                )
                .map_err(|e| e.to_string())?;
            if target_exists && target != source_id {
                connection.execute(
                    "INSERT OR IGNORE INTO relations(id, from_id, to_id, relation_type, created_at) VALUES (?1, ?2, ?3, ?4, ?5)",
                    params![new_id("relation"), source_id, target, format!("field:{key}"), now()],
                ).map_err(|e| e.to_string())?;
            }
        }
    }
    Ok(())
}

#[tauri::command]
fn initialize_database(app: AppHandle) -> Result<Value, String> {
    let connection = db(&app)?;
    Ok(
        json!({"ok": true, "entities": ENTITIES, "database": data_dir(&app)?.join("jason-os.sqlite3"), "integrity": relationship_integrity(&connection)?}),
    )
}

#[tauri::command]
fn check_relationship_integrity(app: AppHandle) -> Result<Value, String> {
    relationship_integrity(&db(&app)?)
}

#[tauri::command]
fn list_records(app: AppHandle, entity: String) -> Result<Vec<Value>, String> {
    if entity != "all" && !is_entity(&entity) {
        return Err("Unknown entity".into());
    }
    let connection = db(&app)?;
    let sql = if entity == "all" {
        "SELECT id, entity, data_json, created_at, updated_at FROM records WHERE archived_at IS NULL AND deleted_at IS NULL ORDER BY updated_at DESC"
    } else {
        "SELECT id, entity, data_json, created_at, updated_at FROM records WHERE entity = ?1 AND archived_at IS NULL AND deleted_at IS NULL ORDER BY updated_at DESC"
    };
    let mut statement = connection.prepare(sql).map_err(|e| e.to_string())?;
    let map_row = |row: &rusqlite::Row<'_>| -> rusqlite::Result<Value> {
        Ok(value_to_record(
            row.get(0)?,
            row.get(1)?,
            row.get(2)?,
            row.get(3)?,
            row.get(4)?,
        ))
    };
    let rows = if entity == "all" {
        statement.query_map([], map_row)
    } else {
        statement.query_map(params![entity], map_row)
    }
    .map_err(|e| e.to_string())?;
    rows.collect::<Result<Vec<_>, _>>()
        .map_err(|e| e.to_string())
}

#[tauri::command]
fn get_record(app: AppHandle, id: String) -> Result<Option<Value>, String> {
    record_by_id(&db(&app)?, &id)
}

#[tauri::command]
fn save_record(app: AppHandle, entity: String, data: Value) -> Result<Value, String> {
    if !is_entity(&entity) {
        return Err("Unknown entity".into());
    }
    if entity == "timelineEvents" {
        return Err("时间线事件只能由系统生成，不能手动创建或修改".into());
    }
    let id = data
        .get("id")
        .and_then(Value::as_str)
        .filter(|s| !s.is_empty())
        .map(str::to_string)
        .unwrap_or_else(|| new_id(&entity));
    let connection = db(&app)?;
    let existing = record_by_id(&connection, &id)?;
    ensure_external_source_capacity(&connection, &entity, &id, &data)?;
    let created_at = existing
        .as_ref()
        .and_then(|record| record.get("createdAt"))
        .and_then(Value::as_str)
        .or_else(|| data.get("createdAt").and_then(Value::as_str))
        .map(str::to_string)
        .unwrap_or_else(now);
    let normalized = normalize_record(&connection, &entity, &data, true)?;
    let enriched = apply_timeline_metadata(&entity, &normalized, &created_at, &now());
    if entity == "timeLogs" && enriched["isRunning"].as_bool().unwrap_or(false) {
        let another_running: bool = connection.query_row(
            "SELECT EXISTS(SELECT 1 FROM records WHERE entity='timeLogs' AND id<>?1 AND archived_at IS NULL AND deleted_at IS NULL AND json_extract(data_json,'$.isRunning')=1)",
            params![id],
            |row| row.get(0),
        ).map_err(|error| error.to_string())?;
        if another_running {
            return Err("全局已有正在运行的计时器".into());
        }
    }
    let saved = write_record(&connection, &entity, &id, &enriched, Some(&created_at))?;
    sync_relations(&connection, &id, &enriched)?;
    if entity == "projects" || entity == "tasks" {
        cascade_context(&connection, &entity, &id)?;
    }
    write_timeline_change_events(&connection, &entity, &id, existing.as_ref(), &saved)?;
    Ok(saved)
}

#[tauri::command]
fn delete_record(app: AppHandle, id: String) -> Result<(), String> {
    archive_record(app, id)
}

#[tauri::command]
fn archive_record(app: AppHandle, id: String) -> Result<(), String> {
    let connection = db(&app)?;
    let Some(mut data) = record_by_id(&connection, &id)? else {
        return Err("记录不存在".into());
    };
    let archived_at = now();
    if let Some(object) = data.as_object_mut() {
        object.insert("archivedAt".into(), Value::String(archived_at.clone()));
    }
    let entity = data["entity"].as_str().unwrap_or("").to_string();
    let created = data["createdAt"].as_str().map(str::to_string);
    write_record(&connection, &entity, &id, &data, created.as_deref())?;
    connection
        .execute(
            "UPDATE records SET archived_at=?2 WHERE id=?1",
            params![id, archived_at],
        )
        .map_err(|e| e.to_string())?;
    connection
        .execute("DELETE FROM records_fts WHERE id=?1", params![id])
        .map_err(|e| e.to_string())?;
    Ok(())
}

#[tauri::command]
fn restore_record(app: AppHandle, id: String) -> Result<Value, String> {
    let connection = db(&app)?;
    let (entity, raw, created): (String, String, String) = connection
        .query_row(
            "SELECT entity, data_json, created_at FROM records WHERE id=?1",
            params![id],
            |row| Ok((row.get(0)?, row.get(1)?, row.get(2)?)),
        )
        .map_err(|_| "记录不存在".to_string())?;
    let mut data: Value = serde_json::from_str(&raw).map_err(|e| e.to_string())?;
    if let Some(object) = data.as_object_mut() {
        object.remove("archivedAt");
        object.remove("deletedAt");
    }
    connection
        .execute(
            "UPDATE records SET archived_at=NULL, deleted_at=NULL WHERE id=?1",
            params![id],
        )
        .map_err(|e| e.to_string())?;
    let saved = write_record(&connection, &entity, &id, &data, Some(&created))?;
    sync_relations(&connection, &id, &data)?;
    Ok(saved)
}

#[tauri::command]
fn list_archived(app: AppHandle) -> Result<Vec<Value>, String> {
    let connection = db(&app)?;
    let mut statement = connection.prepare(
        "SELECT id, entity, data_json, created_at, updated_at FROM records WHERE archived_at IS NOT NULL AND deleted_at IS NULL ORDER BY archived_at DESC"
    ).map_err(|e| e.to_string())?;
    let rows = statement
        .query_map([], |row| {
            Ok(value_to_record(
                row.get(0)?,
                row.get(1)?,
                row.get(2)?,
                row.get(3)?,
                row.get(4)?,
            ))
        })
        .map_err(|e| e.to_string())?;
    rows.collect::<Result<Vec<_>, _>>()
        .map_err(|e| e.to_string())
}

fn search_filtered(
    connection: &Connection,
    query: &str,
    entities: &[String],
) -> Result<Vec<Value>, String> {
    let trimmed = query.trim();
    let mut results = Vec::new();
    if trimmed.is_empty() {
        let mut statement = connection.prepare(
            "SELECT id, entity, data_json, created_at, updated_at FROM records WHERE archived_at IS NULL AND deleted_at IS NULL ORDER BY updated_at DESC LIMIT 80"
        ).map_err(|e| e.to_string())?;
        let rows = statement
            .query_map([], |row| {
                Ok(value_to_record(
                    row.get(0)?,
                    row.get(1)?,
                    row.get(2)?,
                    row.get(3)?,
                    row.get(4)?,
                ))
            })
            .map_err(|e| e.to_string())?;
        for row in rows {
            results.push(row.map_err(|e| e.to_string())?);
        }
    } else {
        let sanitized = trimmed.replace('"', "");
        let fts_query = sanitized
            .split_whitespace()
            .filter(|value| !value.is_empty())
            .map(|value| format!("\"{}\"*", value.replace(['*', ':', '(', ')'], "")))
            .collect::<Vec<_>>()
            .join(" ");
        if !fts_query.is_empty() {
            let mut statement = connection.prepare(
                "SELECT r.id, r.entity, r.data_json, r.created_at, r.updated_at FROM records_fts f JOIN records r ON r.id=f.id WHERE records_fts MATCH ?1 AND r.archived_at IS NULL AND r.deleted_at IS NULL ORDER BY rank LIMIT 80"
            ).map_err(|e| e.to_string())?;
            let rows = statement
                .query_map(params![fts_query], |row| {
                    Ok(value_to_record(
                        row.get(0)?,
                        row.get(1)?,
                        row.get(2)?,
                        row.get(3)?,
                        row.get(4)?,
                    ))
                })
                .map_err(|e| e.to_string())?;
            for row in rows {
                results.push(row.map_err(|e| e.to_string())?);
            }
        }
        if results.is_empty() {
            let pattern = format!("%{}%", trimmed);
            let mut statement = connection.prepare(
                "SELECT id, entity, data_json, created_at, updated_at FROM records WHERE archived_at IS NULL AND deleted_at IS NULL AND (title LIKE ?1 OR body LIKE ?1) ORDER BY updated_at DESC LIMIT 80"
            ).map_err(|e| e.to_string())?;
            let rows = statement
                .query_map(params![pattern], |row| {
                    Ok(value_to_record(
                        row.get(0)?,
                        row.get(1)?,
                        row.get(2)?,
                        row.get(3)?,
                        row.get(4)?,
                    ))
                })
                .map_err(|e| e.to_string())?;
            for row in rows {
                results.push(row.map_err(|e| e.to_string())?);
            }
        }
    }
    if entities.is_empty() {
        return Ok(results);
    }
    Ok(results
        .into_iter()
        .filter(|record| {
            record["entity"]
                .as_str()
                .map(|e| entities.iter().any(|selected| selected == e))
                .unwrap_or(false)
        })
        .collect())
}

#[tauri::command]
fn search_records(app: AppHandle, query: String) -> Result<Vec<Value>, String> {
    search_filtered(&db(&app)?, &query, &[])
}

#[tauri::command]
fn search_records_filtered(
    app: AppHandle,
    query: String,
    entities: Vec<String>,
) -> Result<Vec<Value>, String> {
    search_filtered(&db(&app)?, &query, &entities)
}

#[tauri::command]
fn list_relations(app: AppHandle, id: String) -> Result<Vec<Value>, String> {
    let connection = db(&app)?;
    let mut statement = connection.prepare(
        "SELECT relation_type, from_id, to_id FROM relations WHERE from_id=?1 OR to_id=?1 ORDER BY created_at DESC"
    ).map_err(|e| e.to_string())?;
    let rows = statement
        .query_map(params![id], |row| {
            Ok((
                row.get::<_, String>(0)?,
                row.get::<_, String>(1)?,
                row.get::<_, String>(2)?,
            ))
        })
        .map_err(|e| e.to_string())?;
    let mut related = Vec::new();
    for row in rows {
        let (relation_type, from_id, to_id) = row.map_err(|e| e.to_string())?;
        let related_id = if from_id == id { &to_id } else { &from_id };
        if let Some(mut record) = record_by_id(&connection, related_id)? {
            if let Some(object) = record.as_object_mut() {
                object.insert("relationType".into(), Value::String(relation_type));
                object.insert(
                    "relationDirection".into(),
                    Value::String(
                        if from_id == id {
                            "outgoing"
                        } else {
                            "incoming"
                        }
                        .into(),
                    ),
                );
            }
            related.push(record);
        }
    }
    Ok(related)
}

#[tauri::command]
fn add_relation(
    app: AppHandle,
    from_id: String,
    to_id: String,
    relation_type: String,
) -> Result<(), String> {
    if from_id == to_id {
        return Err("不能关联记录自身".into());
    }
    let connection = db(&app)?;
    if record_by_id(&connection, &from_id)?.is_none()
        || record_by_id(&connection, &to_id)?.is_none()
    {
        return Err("关联记录不存在".into());
    }
    connection.execute(
        "INSERT OR IGNORE INTO relations(id, from_id, to_id, relation_type, created_at) VALUES (?1, ?2, ?3, ?4, ?5)",
        params![new_id("relation"), from_id, to_id, relation_type, now()],
    ).map_err(|e| e.to_string())?;
    Ok(())
}

#[tauri::command]
fn stop_timer(
    app: AppHandle,
    id: String,
    end_at: String,
    duration_minutes: i64,
) -> Result<Value, String> {
    let connection = db(&app)?;
    let mut statement = connection
        .prepare("SELECT entity, data_json, created_at FROM records WHERE id=?1")
        .map_err(|e| e.to_string())?;
    let (entity, raw, created): (String, String, String) = statement
        .query_row(params![id], |r| Ok((r.get(0)?, r.get(1)?, r.get(2)?)))
        .map_err(|_| "Timer not found".to_string())?;
    if entity != "timeLogs" {
        return Err("Record is not a time log".into());
    }
    let mut data = serde_json::from_str::<Value>(&raw).map_err(|e| e.to_string())?;
    let object = data.as_object_mut().ok_or("Invalid timer data")?;
    object.insert("endAt".into(), Value::String(end_at));
    object.insert(
        "durationMinutes".into(),
        Value::Number(duration_minutes.into()),
    );
    object.insert("isRunning".into(), Value::Bool(false));
    object.insert("status".into(), Value::String("completed".into()));
    let saved = write_record(&connection, &entity, &id, &data, Some(&created))?;
    sync_relations(&connection, &id, &data)?;
    Ok(saved)
}

fn all_records(connection: &Connection) -> Result<Vec<Value>, String> {
    let mut statement = connection.prepare("SELECT id, entity, data_json, created_at, updated_at FROM records WHERE deleted_at IS NULL ORDER BY created_at ASC").map_err(|e| e.to_string())?;
    let rows = statement
        .query_map([], |row| {
            Ok(value_to_record(
                row.get(0)?,
                row.get(1)?,
                row.get(2)?,
                row.get(3)?,
                row.get(4)?,
            ))
        })
        .map_err(|e| e.to_string())?;
    rows.collect::<Result<Vec<_>, _>>()
        .map_err(|e| e.to_string())
}

#[tauri::command]
fn export_data(app: AppHandle, format: String) -> Result<String, String> {
    if !["json", "markdown", "csv"].contains(&format.as_str()) {
        return Err("Unsupported export format".into());
    }
    let connection = db(&app)?;
    let records = all_records(&connection)?;
    let target = data_dir(&app)?
        .join("exports")
        .join(format!("jason-os-{}.{}", now(), format));
    let content = match format.as_str() {
        "json" => serde_json::to_string_pretty(
            &json!({"version": 1, "exportedAt": now(), "records": records}),
        )
        .map_err(|e| e.to_string())?,
        "markdown" => records
            .iter()
            .map(|record| {
                format!(
                    "## {} · {}\n\n{}\n",
                    record["entity"].as_str().unwrap_or("record"),
                    record["title"].as_str().unwrap_or("Untitled"),
                    serde_json::to_string_pretty(record).unwrap_or_default()
                )
            })
            .collect::<Vec<_>>()
            .join("\n"),
        _ => {
            let mut out = String::from("id,entity,title,createdAt,updatedAt,data\n");
            for r in records {
                out.push_str(&format!(
                    "\"{}\",\"{}\",\"{}\",\"{}\",\"{}\",\"{}\"\n",
                    r["id"].as_str().unwrap_or(""),
                    r["entity"].as_str().unwrap_or(""),
                    r["title"].as_str().unwrap_or("").replace('"', "\"\""),
                    r["createdAt"].as_str().unwrap_or(""),
                    r["updatedAt"].as_str().unwrap_or(""),
                    serde_json::to_string(&r)
                        .unwrap_or_default()
                        .replace('"', "\"\"")
                ));
            }
            out
        }
    };
    fs::write(&target, content).map_err(|e| e.to_string())?;
    Ok(target.display().to_string())
}

#[tauri::command]
fn create_backup(app: AppHandle) -> Result<String, String> {
    let target = data_dir(&app)?
        .join("backups")
        .join(format!("jason-os-{}.sqlite3", now()));
    let connection = db(&app)?;
    connection
        .execute("VACUUM INTO ?1", params![target.display().to_string()])
        .map_err(|e| e.to_string())?;
    Ok(target.display().to_string())
}

#[tauri::command]
fn list_backups(app: AppHandle) -> Result<Vec<Value>, String> {
    let dir = data_dir(&app)?.join("backups");
    let mut backups = fs::read_dir(&dir).map_err(|e| e.to_string())?
        .filter_map(Result::ok)
        .filter(|entry| entry.path().extension().and_then(|v| v.to_str()) == Some("sqlite3"))
        .filter_map(|entry| {
            let metadata = entry.metadata().ok()?;
            Some(json!({
                "name": entry.file_name().to_string_lossy(),
                "path": entry.path().display().to_string(),
                "size": metadata.len(),
                "modified": metadata.modified().ok()?.duration_since(UNIX_EPOCH).ok()?.as_millis().to_string()
            }))
        }).collect::<Vec<_>>();
    backups.sort_by(|a, b| b["modified"].as_str().cmp(&a["modified"].as_str()));
    Ok(backups)
}

#[tauri::command]
fn restore_backup(app: AppHandle, path: String) -> Result<(), String> {
    let backups_dir = data_dir(&app)?
        .join("backups")
        .canonicalize()
        .map_err(|e| e.to_string())?;
    let source = PathBuf::from(&path)
        .canonicalize()
        .map_err(|e| e.to_string())?;
    if !source.starts_with(&backups_dir) {
        return Err("只能恢复 Jason OS 备份目录中的文件".into());
    }
    let target = data_dir(&app)?.join("jason-os.sqlite3");
    let safety = data_dir(&app)?
        .join("backups")
        .join(format!("before-restore-{}.sqlite3", now()));
    {
        let connection = db(&app)?;
        connection
            .execute("VACUUM INTO ?1", params![safety.display().to_string()])
            .map_err(|e| e.to_string())?;
    }
    for suffix in ["-wal", "-shm"] {
        let _ = fs::remove_file(format!("{}{}", target.display(), suffix));
    }
    fs::copy(source, target).map_err(|e| e.to_string())?;
    Ok(())
}

fn setting(connection: &Connection, key: &str) -> Result<Option<String>, String> {
    connection
        .query_row(
            "SELECT value FROM settings WHERE key = ?1",
            params![key],
            |row| row.get(0),
        )
        .optional()
        .map_err(|e| e.to_string())
}

fn provider_models(provider: &str) -> &'static [&'static str] {
    match provider {
        "deepseek" => &["deepseek-v4-pro", "deepseek-v4-flash"],
        "minimax" => &["MiniMax-M3"],
        "volc-agent-plan" => &["kimi-k3"],
        _ => &["gpt-5.5"],
    }
}

fn ensure_external_source_capacity(
    connection: &Connection,
    entity: &str,
    id: &str,
    data: &Value,
) -> Result<(), String> {
    if entity != "externalSources"
        || data
            .get("status")
            .and_then(Value::as_str)
            .unwrap_or("active")
            != "active"
    {
        return Ok(());
    }
    let active_sources: i64 = connection
        .query_row(
            "SELECT COUNT(*) FROM records WHERE entity='externalSources' AND id<>?1 AND archived_at IS NULL AND deleted_at IS NULL AND COALESCE(json_extract(data_json,'$.status'),'active')='active'",
            params![id],
            |row| row.get(0),
        )
        .map_err(|error| error.to_string())?;
    if active_sources >= 20 {
        return Err("第一版最多启用 20 个情报源；请先暂停不需要的 Source".into());
    }
    Ok(())
}

fn capture_provider_config(configured: bool) -> Value {
    json!({
        "providers": [{
            "id":"redfox", "label":"RedFoxHub", "configured":configured,
            "supportedPlatforms":["微信公众号","抖音","小红书"],
            "automaticSync":false, "mediaDownload":false
        }]
    })
}

fn redfox_key() -> Result<String, String> {
    if let Ok(value) = std::env::var("REDFOX_API_KEY") {
        if !value.trim().is_empty() {
            return Ok(value);
        }
    }
    read_secrets()?
        .get("redfox")
        .and_then(Value::as_str)
        .filter(|value| !value.trim().is_empty())
        .map(str::to_string)
        .ok_or("尚未配置 RedFox API Key".to_string())
}

fn timestamp_after_days(value: &str, days: i64) -> String {
    let base = value.parse::<i64>().unwrap_or_default();
    (base + days * 86_400_000).to_string()
}

fn existing_inbox_capture(
    connection: &Connection,
    source_url: &str,
    canonical_url: &str,
) -> Result<Option<Value>, String> {
    connection.query_row(
        "SELECT id,entity,data_json,created_at,updated_at FROM records WHERE entity='inbox' AND archived_at IS NULL AND deleted_at IS NULL AND (json_extract(data_json,'$.sourceUrl')=?1 OR json_extract(data_json,'$.canonicalUrl')=?2) LIMIT 1",
        params![source_url, canonical_url],
        |row| Ok(value_to_record(row.get(0)?, row.get(1)?, row.get(2)?, row.get(3)?, row.get(4)?)),
    ).optional().map_err(|error| error.to_string())
}

#[tauri::command]
fn get_capture_provider_config() -> Result<Value, String> {
    Ok(capture_provider_config(redfox_key().is_ok()))
}

#[tauri::command]
fn configure_capture_provider(provider: String, api_key: String) -> Result<Value, String> {
    if provider != "redfox" {
        return Err("当前只支持配置 RedFoxHub".into());
    }
    if !api_key.trim().is_empty() {
        store_provider_key("redfox", api_key.trim())?;
    } else if redfox_key().is_err() {
        return Err("请输入 RedFox API Key".into());
    }
    get_capture_provider_config()
}

#[tauri::command]
async fn test_capture_provider(provider: String, url: String) -> Result<Value, String> {
    if provider != "redfox" {
        return Err("当前只支持测试 RedFoxHub".into());
    }
    tauri::async_runtime::spawn_blocking(move || {
        let started = Instant::now();
        let capture = redfox::capture(&redfox_key()?, url.trim())?;
        Ok(json!({"ok":true,"provider":"redfox","latencyMs":started.elapsed().as_millis(),"content":capture.canonical}))
    }).await.map_err(|error| format!("采集服务后台任务失败：{error}"))?
}

#[tauri::command]
fn list_external_items(app: AppHandle, limit: Option<i64>) -> Result<Vec<Value>, String> {
    external_intelligence::list_items(&db(&app)?, limit.unwrap_or(80))
}

#[tauri::command]
fn cleanup_external_cache(app: AppHandle) -> Result<Value, String> {
    let connection = db(&app)?;
    let current = now();
    let raw_dir = data_dir(&app)?
        .join("external-intelligence/raw")
        .canonicalize()
        .map_err(|error| error.to_string())?;
    let paths = {
        let mut statement = connection.prepare("SELECT raw_payload_path FROM external_items WHERE expires_at IS NOT NULL AND CAST(expires_at AS INTEGER) < CAST(?1 AS INTEGER) AND raw_payload_path IS NOT NULL").map_err(|error| error.to_string())?;
        let rows = statement
            .query_map(params![current], |row| row.get::<_, String>(0))
            .map_err(|error| error.to_string())?;
        rows.filter_map(Result::ok).collect::<Vec<_>>()
    };
    let removed = external_intelligence::cleanup_expired(&connection, &current)?;
    let mut raw_removed = 0;
    for value in paths {
        let path = PathBuf::from(value);
        if path
            .canonicalize()
            .ok()
            .is_some_and(|canonical| canonical.starts_with(&raw_dir))
            && fs::remove_file(path).is_ok()
        {
            raw_removed += 1;
        }
    }
    Ok(json!({"ok":true,"removed":removed,"rawRemoved":raw_removed}))
}

fn capture_platform(url: &str) -> &'static str {
    let lower = url.to_lowercase();
    if lower.contains("mp.weixin.qq.com") {
        "微信公众号"
    } else if lower.contains("douyin.com") {
        "抖音"
    } else if lower.contains("xiaohongshu.com") || lower.contains("xhslink.com") {
        "小红书"
    } else if lower.contains("twitter.com") || lower.contains("x.com") {
        "X"
    } else if lower.contains("tiktok.com") {
        "TikTok"
    } else if lower.contains("instagram.com") {
        "Instagram"
    } else if lower.contains("facebook.com") || lower.contains("fb.watch") {
        "Facebook"
    } else if lower.contains("reddit.com") || lower.contains("redd.it") {
        "Reddit"
    } else if lower.contains("channels.weixin.qq.com") {
        "视频号"
    } else {
        "网页"
    }
}

fn executable(candidates: &[&str]) -> Option<String> {
    candidates
        .iter()
        .find(|candidate| Path::new(candidate).exists())
        .map(|value| value.to_string())
}

fn run_yt_dlp(url: &str) -> Result<Value, String> {
    let command = executable(&["/opt/homebrew/bin/yt-dlp", "/usr/local/bin/yt-dlp"])
        .ok_or("未安装 yt-dlp")?;
    let output = Command::new(command)
        .args([
            "--dump-single-json",
            "--skip-download",
            "--no-playlist",
            "--socket-timeout",
            "15",
            "--retries",
            "1",
            "--extractor-retries",
            "1",
            "--",
            url,
        ])
        .output()
        .map_err(|error| error.to_string())?;
    if !output.status.success() {
        return Err(String::from_utf8_lossy(&output.stderr).trim().to_string());
    }
    serde_json::from_slice(&output.stdout).map_err(|error| error.to_string())
}

fn run_gallery_dl(url: &str) -> Result<Value, String> {
    let command = executable(&["/opt/homebrew/bin/gallery-dl", "/usr/local/bin/gallery-dl"])
        .ok_or("未安装 gallery-dl")?;
    let output = Command::new(command)
        .args(["--dump-json", "--", url])
        .output()
        .map_err(|error| error.to_string())?;
    if !output.status.success() {
        return Err(String::from_utf8_lossy(&output.stderr).trim().to_string());
    }
    let first = String::from_utf8_lossy(&output.stdout)
        .lines()
        .find(|line| !line.trim().is_empty())
        .unwrap_or("")
        .to_string();
    serde_json::from_str(&first).map_err(|error| error.to_string())
}

fn html_meta(html: &str, key: &str) -> String {
    let escaped = regex::escape(key);
    for pattern in [
        format!(
            r#"(?is)<meta[^>]+(?:property|name)=["']{escaped}["'][^>]+content=["']([^"']*)["']"#
        ),
        format!(
            r#"(?is)<meta[^>]+content=["']([^"']*)["'][^>]+(?:property|name)=["']{escaped}["']"#
        ),
    ] {
        if let Ok(regex) = Regex::new(&pattern) {
            if let Some(value) = regex.captures(html).and_then(|capture| capture.get(1)) {
                return value.as_str().trim().to_string();
            }
        }
    }
    String::new()
}

fn fetch_web_metadata(url: &str) -> Result<Value, String> {
    let client = reqwest::blocking::Client::builder()
        .timeout(Duration::from_secs(20))
        .user_agent("Mozilla/5.0 JasonOS/3.1")
        .build()
        .map_err(|error| error.to_string())?;
    let response = client.get(url).send().map_err(|error| error.to_string())?;
    if !response.status().is_success() {
        return Err(format!("网页返回 {}", response.status()));
    }
    let final_url = response.url().to_string();
    let html = response.text().map_err(|error| error.to_string())?;
    let title = html_meta(&html, "og:title");
    let description = {
        let value = html_meta(&html, "og:description");
        if value.is_empty() {
            html_meta(&html, "description")
        } else {
            value
        }
    };
    Ok(
        json!({"title":title,"description":description,"webpage_url":final_url,"thumbnail":html_meta(&html,"og:image"),"uploader":html_meta(&html,"article:author")}),
    )
}

#[tauri::command]
async fn capture_link(app: AppHandle, url: String) -> Result<Value, String> {
    tauri::async_runtime::spawn_blocking(move || capture_link_blocking(app, url))
        .await
        .map_err(|error| format!("采集后台任务失败：{error}"))?
}

fn capture_link_blocking(app: AppHandle, url: String) -> Result<Value, String> {
    let url = url.trim().to_string();
    if !(url.starts_with("https://") || url.starts_with("http://")) {
        return Err("请输入有效的 http/https 链接".into());
    }
    let platform = capture_platform(&url);
    let capture_id = new_id("capture");
    let captured_at = now();
    let mut errors = Vec::new();
    let mut external_item_id = String::new();
    let mut capture_provider = "local".to_string();
    let mut local_metadata_path = String::new();
    let redfox_result = if redfox::platform_for_url(&url).is_some() {
        match redfox_key() {
            Ok(key) => {
                let endpoint = redfox::request_for_url(&url)
                    .ok()
                    .map(|value| value.0.to_string())
                    .unwrap_or_default();
                match redfox::capture(&key, &url) {
                    Ok(capture) => {
                        let directory = data_dir(&app)?.join("external-intelligence/raw");
                        let raw_path = directory.join(format!("{capture_id}.json"));
                        fs::write(
                            &raw_path,
                            serde_json::to_vec_pretty(&capture.raw)
                                .map_err(|error| error.to_string())?,
                        )
                        .map_err(|error| error.to_string())?;
                        let connection = db(&app)?;
                        external_item_id = external_intelligence::upsert_item(
                            &connection,
                            &new_id("external-item"),
                            &capture.canonical,
                            &captured_at,
                            &timestamp_after_days(&captured_at, 30),
                            &raw_path.to_string_lossy(),
                        )?;
                        external_intelligence::record_provider_call(
                            &connection,
                            &new_id("provider-call"),
                            "redfox",
                            &capture.endpoint,
                            None,
                            &captured_at,
                            true,
                            Some(capture.status_code),
                            1,
                            None,
                        )?;
                        local_metadata_path = raw_path.to_string_lossy().to_string();
                        capture_provider = "redfox".into();
                        Some(capture.canonical)
                    }
                    Err(error) => {
                        if !endpoint.is_empty() {
                            let connection = db(&app)?;
                            external_intelligence::record_provider_call(
                                &connection,
                                &new_id("provider-call"),
                                "redfox",
                                &endpoint,
                                None,
                                &captured_at,
                                false,
                                None,
                                0,
                                Some(&error),
                            )?;
                        }
                        errors.push(format!("RedFox: {error}"));
                        None
                    }
                }
            }
            Err(_) => None,
        }
    } else {
        None
    };
    let (metadata, method) = if let Some(value) = redfox_result {
        (value, "redfox")
    } else {
        match run_yt_dlp(&url) {
            Ok(value) => (value, "yt-dlp"),
            Err(error) => {
                errors.push(format!("yt-dlp: {error}"));
                match run_gallery_dl(&url) {
                    Ok(value) => (value, "gallery-dl"),
                    Err(error) => {
                        errors.push(format!("gallery-dl: {error}"));
                        match fetch_web_metadata(&url) {
                            Ok(value) => (value, "open-graph"),
                            Err(error) => {
                                errors.push(format!("网页读取: {error}"));
                                (json!({"webpage_url":url}), "link-only")
                            }
                        }
                    }
                }
            }
        }
    };
    if local_metadata_path.is_empty() {
        let directory = data_dir(&app)?
            .join("attachments/captures")
            .join(&capture_id);
        fs::create_dir_all(&directory).map_err(|error| error.to_string())?;
        let path = directory.join("metadata.json");
        fs::write(
            &path,
            serde_json::to_vec_pretty(&metadata).map_err(|error| error.to_string())?,
        )
        .map_err(|error| error.to_string())?;
        local_metadata_path = path.to_string_lossy().to_string();
    }
    if external_item_id.is_empty()
        && redfox::platform_for_url(&url).is_some()
        && method != "link-only"
    {
        let mut canonical_item = redfox::normalize_response(&url, method, &metadata);
        if let Some(object) = canonical_item.as_object_mut() {
            object.insert("provider".into(), Value::String(method.into()));
            object.insert("providerEndpoint".into(), Value::String(method.into()));
        }
        external_item_id = external_intelligence::upsert_item(
            &db(&app)?,
            &new_id("external-item"),
            &canonical_item,
            &captured_at,
            &timestamp_after_days(&captured_at, 30),
            &local_metadata_path,
        )?;
        capture_provider = method.into();
    }
    let title = metadata["title"]
        .as_str()
        .filter(|value| !value.trim().is_empty())
        .or_else(|| {
            metadata["fulltitle"]
                .as_str()
                .filter(|value| !value.trim().is_empty())
        })
        .unwrap_or(&url)
        .to_string();
    let text = metadata["description"]
        .as_str()
        .filter(|value| !value.trim().is_empty())
        .or_else(|| {
            metadata["content"]
                .as_str()
                .filter(|value| !value.trim().is_empty())
        })
        .unwrap_or("")
        .to_string();
    let author = metadata["uploader"]
        .as_str()
        .or_else(|| metadata["channel"].as_str())
        .or_else(|| metadata["author"].as_str())
        .unwrap_or("")
        .to_string();
    let canonical = metadata["canonicalUrl"]
        .as_str()
        .or_else(|| metadata["webpage_url"].as_str())
        .or_else(|| metadata["original_url"].as_str())
        .unwrap_or(&url)
        .to_string();
    let connection = db(&app)?;
    let existing = existing_inbox_capture(&connection, &url, &canonical)?;
    let mut record = json!({
        "content": if text.is_empty() { title.clone() } else { format!("{}\n\n{}", title, text) },
        "type":"link", "status":"unprocessed", "platform":platform, "sourceUrl":url,
        "canonicalUrl":canonical, "externalContentId":metadata["externalId"], "contentType":metadata["contentType"],
        "author":author, "title":title, "publishedAt":metadata.get("publishedAt").cloned().unwrap_or_else(|| metadata["upload_date"].clone()),
        "cover":metadata.get("coverUrl").cloned().unwrap_or_else(|| metadata["thumbnail"].clone()),
        "metrics":metadata["metrics"], "captureProvider":capture_provider,
        "captureStatus": if method == "link-only" || (title == url && text.is_empty()) { "link_saved" } else { "captured" },
        "captureMethod":method, "captureErrors":errors, "localMetadataPath":local_metadata_path,
        "externalItemId":external_item_id, "rightsConfirmed":false, "savedAt":captured_at, "lastCapturedAt":now()
    });
    if let Some(existing) = existing {
        if let Some(object) = record.as_object_mut() {
            object.insert("id".into(), existing["id"].clone());
            object.insert("createdAt".into(), existing["createdAt"].clone());
        }
    }
    save_record(app, "inbox".into(), record)
}

fn provider_label(provider: &str) -> &'static str {
    match provider {
        "deepseek" => "DeepSeek",
        "minimax" => "MiniMax Token Plan",
        "volc-agent-plan" => "火山引擎 Agent Plan",
        _ => "HackStart",
    }
}

fn provider_base_url(provider: &str) -> &'static str {
    match provider {
        "deepseek" => "https://api.deepseek.com/chat/completions",
        "minimax" => "https://api.minimaxi.com/anthropic/v1/messages",
        "volc-agent-plan" => "https://ark.cn-beijing.volces.com/api/plan/v3/responses",
        _ => "https://ip2.hackstart.org/v1/chat/completions",
    }
}

fn provider_key_account(provider: &str) -> &'static str {
    match provider {
        "deepseek" => "deepseek-api-key",
        "minimax" => "minimax-token-plan-api-key",
        "volc-agent-plan" => "volc-agent-plan-api-key",
        _ => "hackstart-api-key",
    }
}

fn secrets_path() -> Result<PathBuf, String> {
    let home = std::env::var("HOME").map_err(|_| "无法定位用户目录".to_string())?;
    let dir = PathBuf::from(home).join("Library/Application Support/com.jasonos.desktop");
    fs::create_dir_all(&dir).map_err(|e| e.to_string())?;
    Ok(dir.join("secrets.json"))
}

fn read_secrets() -> Result<Map<String, Value>, String> {
    let path = secrets_path()?;
    if !path.exists() {
        return Ok(Map::new());
    }
    let raw = fs::read_to_string(path).map_err(|e| e.to_string())?;
    serde_json::from_str::<Value>(&raw)
        .map_err(|e| e.to_string())?
        .as_object()
        .cloned()
        .ok_or("凭据文件格式无效".to_string())
}

fn provider_key(provider: &str) -> Result<String, String> {
    let environment_name = match provider {
        "deepseek" => "DEEPSEEK_API_KEY",
        "minimax" => "MINIMAX_TOKEN_PLAN_API_KEY",
        "volc-agent-plan" => "VOLC_AGENT_PLAN_API_KEY",
        _ => "HACKSTART_API_KEY",
    };
    if let Ok(value) = std::env::var(environment_name) {
        if !value.trim().is_empty() {
            return Ok(value);
        }
    }
    read_secrets()?
        .get(provider)
        .and_then(Value::as_str)
        .filter(|value| !value.trim().is_empty())
        .map(str::to_string)
        .ok_or_else(|| {
            format!(
                "尚未配置 {} API Key。请在设置中保存。",
                provider_label(provider)
            )
        })
}

fn store_provider_key(provider: &str, password: &str) -> Result<(), String> {
    let path = secrets_path()?;
    let mut secrets = read_secrets()?;
    secrets.insert(provider.to_string(), Value::String(password.to_string()));
    let temporary = path.with_extension("json.tmp");
    fs::write(
        &temporary,
        serde_json::to_vec_pretty(&Value::Object(secrets)).map_err(|e| e.to_string())?,
    )
    .map_err(|e| e.to_string())?;
    #[cfg(unix)]
    fs::set_permissions(&temporary, fs::Permissions::from_mode(0o600))
        .map_err(|e| e.to_string())?;
    fs::rename(&temporary, &path).map_err(|e| e.to_string())?;
    #[cfg(unix)]
    fs::set_permissions(&path, fs::Permissions::from_mode(0o600)).map_err(|e| e.to_string())?;
    Ok(())
}

fn active_provider(connection: &Connection) -> Result<String, String> {
    let provider = setting(connection, "ai_provider")?.unwrap_or_else(|| "hackstart".to_string());
    if ["hackstart", "deepseek", "minimax", "volc-agent-plan"].contains(&provider.as_str()) {
        Ok(provider)
    } else {
        Ok("hackstart".into())
    }
}

fn selected_model(connection: &Connection, provider: &str) -> Result<String, String> {
    Ok(setting(connection, &format!("{}_model", provider))?
        .or_else(|| {
            if provider == "hackstart" {
                setting(connection, "hackstart_model").ok().flatten()
            } else {
                None
            }
        })
        .unwrap_or_else(|| provider_models(provider)[0].to_string()))
}

fn provider_config(connection: &Connection, provider: &str) -> Result<Value, String> {
    let models = provider_models(provider)
        .iter()
        .map(|model| {
            let description = match *model {
                "deepseek-v4-pro" => "高质量复杂推理",
                "deepseek-v4-flash" => "低延迟高性价比",
                "MiniMax-M3" => "最新旗舰 Agent 与推理模型",
                "kimi-k3" => "Agent Plan Medium 当前文本模型",
                _ => "HackStart 模型",
            };
            json!({"id": model, "label": model, "description": description})
        })
        .collect::<Vec<_>>();
    Ok(json!({
        "id": provider,
        "label": provider_label(provider),
        "configured": provider_key(provider).is_ok(),
        "baseUrl": provider_base_url(provider),
        "model": selected_model(connection, provider)?,
        "models": models
    }))
}

fn is_external_url(url: &str) -> bool {
    url.starts_with("https://") || url.starts_with("http://")
}

#[tauri::command]
fn open_external(url: String) -> Result<(), String> {
    if !is_external_url(&url) {
        return Err("只允许打开 http 或 https 新闻链接".into());
    }
    Command::new("open")
        .arg(url)
        .status()
        .map_err(|error| error.to_string())?;
    Ok(())
}

#[tauri::command]
fn get_ai_config(app: AppHandle) -> Result<Value, String> {
    let connection = db(&app)?;
    let provider = active_provider(&connection)?;
    let providers = ["hackstart", "deepseek", "minimax", "volc-agent-plan"]
        .iter()
        .map(|item| provider_config(&connection, item))
        .collect::<Result<Vec<_>, _>>()?;
    let current = provider_config(&connection, &provider)?;
    Ok(json!({
        "provider": provider,
        "providerLabel": current["label"],
        "configured": current["configured"],
        "model": current["model"],
        "baseUrl": current["baseUrl"],
        "providers": providers
    }))
}

fn provider_response_text(provider: &str, response: &Value) -> String {
    if provider == "volc-agent-plan" {
        response["output"]
            .as_array()
            .into_iter()
            .flatten()
            .flat_map(|item| item["content"].as_array().into_iter().flatten())
            .filter(|content| content["type"] == "output_text")
            .filter_map(|content| content["text"].as_str())
            .collect::<Vec<_>>()
            .join("")
    } else if provider == "minimax" {
        response["content"]
            .as_array()
            .into_iter()
            .flatten()
            .filter(|content| content["type"] == "text")
            .filter_map(|content| content["text"].as_str())
            .collect::<Vec<_>>()
            .join("")
    } else {
        response["choices"][0]["message"]["content"]
            .as_str()
            .unwrap_or("")
            .to_string()
    }
}

fn call_provider(
    provider: &str,
    key: &str,
    model: &str,
    system: &str,
    messages: &[Value],
    max_tokens: u64,
) -> Result<(Value, u128), String> {
    let client = reqwest::blocking::Client::builder()
        .timeout(Duration::from_secs(90))
        .build()
        .map_err(|e| e.to_string())?;
    let started = Instant::now();
    let (request, payload) = if provider == "volc-agent-plan" {
        let payload = json!({"model": model, "instructions": system, "input": messages, "max_output_tokens": max_tokens, "thinking": {"type": "disabled"}});
        (
            client.post(provider_base_url(provider)).bearer_auth(key),
            payload,
        )
    } else if provider == "minimax" {
        let payload = json!({"model": model, "system": system, "messages": messages, "max_tokens": max_tokens});
        (
            client
                .post(provider_base_url(provider))
                .header("x-api-key", key)
                .header("anthropic-version", "2023-06-01"),
            payload,
        )
    } else {
        let mut chat_messages = vec![json!({"role": "system", "content": system})];
        chat_messages.extend(messages.iter().cloned());
        let mut payload = json!({"model": model, "temperature": 0.2, "messages": chat_messages, "max_tokens": max_tokens});
        if provider == "deepseek" {
            payload["thinking"] = json!({"type": "disabled"});
        }
        (
            client.post(provider_base_url(provider)).bearer_auth(key),
            payload,
        )
    };
    let response = request
        .json(&payload)
        .send()
        .map_err(|e| format!("{} 请求失败：{e}", provider_label(provider)))?;
    let latency = started.elapsed().as_millis();
    let status = response.status();
    let response_json: Value = response
        .json()
        .map_err(|e| format!("无法解析 {} 响应：{e}", provider_label(provider)))?;
    if !status.is_success() {
        let message = response_json["error"]["message"]
            .as_str()
            .or_else(|| response_json["base_resp"]["status_msg"].as_str())
            .or_else(|| response_json["message"].as_str())
            .unwrap_or("未知错误");
        return Err(format!(
            "{} 返回 {}：{}",
            provider_label(provider),
            status,
            message
        ));
    }
    Ok((response_json, latency))
}

fn verify_provider(provider: &str, key: &str, model: &str) -> Result<Value, String> {
    let messages =
        vec![json!({"role": "user", "content": "Reply exactly: JASON_OS_CONNECTION_OK"})];
    let (response, latency_ms) = call_provider(
        provider,
        key,
        model,
        "You are a connectivity test. Return only the requested token.",
        &messages,
        64,
    )?;
    let content = provider_response_text(provider, &response);
    if content.trim().is_empty() {
        return Err(format!("{} 已响应但未返回文本", provider_label(provider)));
    }
    Ok(
        json!({"ok": true, "provider": provider, "model": model, "latencyMs": latency_ms, "content": content}),
    )
}

#[tauri::command]
fn test_ai_provider(app: AppHandle, provider: String, model: String) -> Result<Value, String> {
    if !["hackstart", "deepseek", "minimax", "volc-agent-plan"].contains(&provider.as_str()) {
        return Err("不支持的 AI 服务商".into());
    }
    let connection = db(&app)?;
    let selected = if model.trim().is_empty() {
        selected_model(&connection, &provider)?
    } else {
        model
    };
    verify_provider(&provider, &provider_key(&provider)?, &selected)
}

#[tauri::command]
fn configure_ai_provider(
    app: AppHandle,
    provider: String,
    api_key: String,
    model: String,
) -> Result<Value, String> {
    if !["hackstart", "deepseek", "minimax", "volc-agent-plan"].contains(&provider.as_str()) {
        return Err("不支持的 AI 服务商".into());
    }
    let chosen_model = if model.trim().is_empty() {
        provider_models(&provider)[0]
    } else {
        model.trim()
    };
    if provider != "hackstart" && !provider_models(&provider).contains(&chosen_model) {
        return Err("模型不在当前支持列表中".into());
    }
    let candidate_key = if api_key.trim().is_empty() {
        provider_key(&provider)?
    } else {
        api_key.trim().to_string()
    };
    verify_provider(&provider, &candidate_key, chosen_model)?;
    if !api_key.trim().is_empty() {
        store_provider_key(&provider, api_key.trim())?;
    }
    let connection = db(&app)?;
    let timestamp = now();
    connection.execute(
        "INSERT INTO settings(key,value,updated_at) VALUES ('ai_provider',?1,?2) ON CONFLICT(key) DO UPDATE SET value=excluded.value,updated_at=excluded.updated_at",
        params![provider, timestamp],
    ).map_err(|e| e.to_string())?;
    connection.execute(
        "INSERT INTO settings(key,value,updated_at) VALUES (?1,?2,?3) ON CONFLICT(key) DO UPDATE SET value=excluded.value,updated_at=excluded.updated_at",
        params![format!("{}_model", provider), chosen_model, now()],
    ).map_err(|e| e.to_string())?;
    get_ai_config(app)
}

// Backward-compatible commands for existing installations.
#[tauri::command]
fn get_hackstart_config(app: AppHandle) -> Result<Value, String> {
    get_ai_config(app)
}

#[tauri::command]
fn configure_hackstart(app: AppHandle, api_key: String, model: String) -> Result<Value, String> {
    configure_ai_provider(app, "hackstart".into(), api_key, model)
}

fn agent_tool_metadata(tool: &str) -> Option<(&'static str, &'static str, bool, &'static str)> {
    match tool {
        "createMentalModel" => Some(("mentalModels", "LOW_WRITE", true, "CREATE")),
        "updateMentalModel" => Some(("mentalModels", "MEDIUM_WRITE", true, "UPDATE")),
        "createKnowledge" => Some(("knowledge", "LOW_WRITE", true, "CREATE")),
        "updateKnowledge" => Some(("knowledge", "MEDIUM_WRITE", true, "UPDATE")),
        "createGoal" => Some(("goals", "LOW_WRITE", true, "CREATE")),
        "updateGoal" => Some(("goals", "MEDIUM_WRITE", true, "UPDATE")),
        "createProject" => Some(("projects", "LOW_WRITE", true, "CREATE")),
        "updateProject" => Some(("projects", "MEDIUM_WRITE", true, "UPDATE")),
        "createTask" => Some(("tasks", "LOW_WRITE", true, "CREATE")),
        "updateTask" => Some(("tasks", "MEDIUM_WRITE", true, "UPDATE")),
        "completeTask" => Some(("tasks", "MEDIUM_WRITE", true, "COMPLETE")),
        "startTimer" => Some(("timeLogs", "LOW_WRITE", true, "START_TIMER")),
        "stopTimer" => Some(("timeLogs", "MEDIUM_WRITE", true, "STOP_TIMER")),
        "createTimeRecord" => Some(("timeLogs", "LOW_WRITE", true, "CREATE")),
        "createDecision" => Some(("decisions", "MEDIUM_WRITE", true, "CREATE")),
        "updateDecision" => Some(("decisions", "MEDIUM_WRITE", true, "UPDATE")),
        "createReview" => Some(("reviews", "LOW_WRITE", true, "CREATE")),
        "createInsight" => Some(("insights", "LOW_WRITE", true, "CREATE")),
        "createPrinciple" => Some(("principles", "LOW_WRITE", true, "CREATE")),
        "createExternalSource" => Some(("externalSources", "MEDIUM_WRITE", true, "CREATE")),
        "updateExternalSource" => Some(("externalSources", "MEDIUM_WRITE", true, "UPDATE")),
        "updateExternalSignal" => Some(("signals", "MEDIUM_WRITE", true, "UPDATE")),
        "createOpportunity" => Some(("opportunities", "LOW_WRITE", true, "CREATE")),
        "updateOpportunity" => Some(("opportunities", "MEDIUM_WRITE", true, "UPDATE")),
        _ => None,
    }
}

fn extract_json_value(text: &str) -> Option<Value> {
    serde_json::from_str(text.trim()).ok().or_else(|| {
        let start = text.find('{')?;
        let end = text.rfind('}')?;
        serde_json::from_str(&text[start..=end]).ok()
    })
}

fn action_idempotency_key(tool: &str, input: &Value) -> String {
    let mut hasher = std::collections::hash_map::DefaultHasher::new();
    tool.hash(&mut hasher);
    input.to_string().hash(&mut hasher);
    format!("agent:{:016x}", hasher.finish())
}

fn required_agent_fields(entity: &str) -> &'static [&'static str] {
    match entity {
        "mentalModels" => &["name"],
        "knowledge" => &["title", "content"],
        "goals" | "projects" | "tasks" | "decisions" | "reviews" => &["title"],
        "insights" | "principles" => &["statement"],
        "externalSources" => &["name"],
        "signals" | "opportunities" | "intelligenceBriefs" => &["title"],
        "timeLogs" => &["title"],
        _ => &[],
    }
}

fn validate_agent_input(entity: &str, action_type: &str, input: &Value) -> Result<(), String> {
    let object = input.as_object().ok_or("AI Action 参数必须是结构化对象")?;
    if ["UPDATE", "COMPLETE"].contains(&action_type)
        && object
            .get("id")
            .and_then(Value::as_str)
            .unwrap_or("")
            .is_empty()
    {
        return Err("更新操作缺少真实记录 ID".into());
    }
    if action_type == "CREATE" {
        for field in required_agent_fields(entity) {
            if object
                .get(*field)
                .and_then(Value::as_str)
                .unwrap_or("")
                .trim()
                .is_empty()
            {
                return Err(format!("缺少必要字段：{field}"));
            }
        }
    }
    Ok(())
}

fn context_string(context: &Value, key: &str) -> Option<String> {
    context
        .get(key)
        .and_then(Value::as_str)
        .map(str::trim)
        .filter(|value| !value.is_empty())
        .map(str::to_string)
}

fn timeline_analysis_is_read_only(context: &Value) -> bool {
    context.get("analysisMode").and_then(Value::as_str) == Some("timeline_readonly")
}

fn apply_agent_context(
    connection: &Connection,
    entity: &str,
    action_type: &str,
    input: &Value,
    context: &Value,
) -> Result<Value, String> {
    let mut resolved = input.clone();
    if !resolved.is_object() {
        resolved = json!({});
    }
    let object = resolved.as_object_mut().unwrap();
    if entity == "projects"
        && object
            .get("goalId")
            .and_then(Value::as_str)
            .unwrap_or("")
            .is_empty()
    {
        if let Some(goal_id) = context_string(context, "currentGoalId") {
            if valid_reference(connection, &goal_id, "goals", true)?.is_some() {
                object.insert("goalId".into(), Value::String(goal_id));
            }
        }
    }
    if entity == "tasks" {
        if object
            .get("projectId")
            .and_then(Value::as_str)
            .unwrap_or("")
            .is_empty()
        {
            if let Some(project_id) = context_string(context, "currentProjectId") {
                if valid_reference(connection, &project_id, "projects", true)?.is_some() {
                    object.insert("projectId".into(), Value::String(project_id));
                }
            }
        }
        if object
            .get("goalId")
            .and_then(Value::as_str)
            .unwrap_or("")
            .is_empty()
            && object.get("projectId").is_none()
        {
            if let Some(goal_id) = context_string(context, "currentGoalId") {
                if valid_reference(connection, &goal_id, "goals", true)?.is_some() {
                    object.insert("goalId".into(), Value::String(goal_id));
                }
            }
        }
        if action_type == "COMPLETE" && object.get("id").is_none() {
            if let Some(task_id) = context_string(context, "currentTaskId") {
                object.insert("id".into(), Value::String(task_id));
            }
        }
    }
    if ["externalSources", "opportunities"].contains(&entity) {
        if object
            .get("projectId")
            .and_then(Value::as_str)
            .unwrap_or("")
            .is_empty()
        {
            if let Some(project_id) = context_string(context, "currentProjectId") {
                object.insert("projectId".into(), Value::String(project_id));
            }
        }
        if object
            .get("goalId")
            .and_then(Value::as_str)
            .unwrap_or("")
            .is_empty()
        {
            if let Some(goal_id) = context_string(context, "currentGoalId") {
                object.insert("goalId".into(), Value::String(goal_id));
            }
        }
    }

    if entity == "timeLogs"
        && object
            .get("title")
            .and_then(Value::as_str)
            .unwrap_or("")
            .is_empty()
    {
        let title = [
            ("currentTaskId", "tasks"),
            ("currentProjectId", "projects"),
            ("currentGoalId", "goals"),
        ]
        .iter()
        .find_map(|(context_key, expected)| {
            let id = context_string(context, context_key)?;
            valid_reference(connection, &id, expected, false)
                .ok()
                .flatten()
                .and_then(|record| {
                    record
                        .get("title")
                        .or_else(|| record.get("name"))
                        .and_then(Value::as_str)
                        .map(str::to_string)
                })
        })
        .unwrap_or_else(|| "专注工作".into());
        object.insert("title".into(), Value::String(title));
    }
    if ["timeLogs", "decisions", "reviews"].contains(&entity) {
        for (field, context_key, expected) in [
            ("taskId", "currentTaskId", "tasks"),
            ("projectId", "currentProjectId", "projects"),
            ("goalId", "currentGoalId", "goals"),
        ] {
            if object
                .get(field)
                .and_then(Value::as_str)
                .unwrap_or("")
                .is_empty()
            {
                if let Some(id) = context_string(context, context_key) {
                    if valid_reference(connection, &id, expected, true)?.is_some() {
                        object.insert(field.into(), Value::String(id));
                    }
                }
            }
        }
    }
    if entity == "mentalModels" {
        let pairs = [
            ("definition", "corePrinciple"),
            ("trigger", "useCases"),
            ("questions", "keyQuestions"),
            ("method", "steps"),
            ("application", "useCases"),
        ];
        for (legacy, current) in pairs {
            if object
                .get(legacy)
                .and_then(Value::as_str)
                .unwrap_or("")
                .is_empty()
            {
                if let Some(value) = object
                    .get(current)
                    .and_then(Value::as_str)
                    .map(str::to_string)
                {
                    object.insert(legacy.into(), Value::String(value));
                }
            }
        }
    }
    Ok(resolved)
}

fn preview_label(key: &str) -> &str {
    match key {
        "name" => "名称",
        "category" => "类型",
        "corePrinciple" => "核心原则",
        "problem" => "解决的问题",
        "framework" => "框架",
        "steps" => "流程",
        "keyQuestions" => "核心问题",
        "useCases" => "适用场景",
        "outputTemplate" => "输出模板",
        "title" => "名称",
        "description" => "说明",
        "dueDate" => "截止日期",
        "goalId" => "目标",
        "projectId" => "项目",
        "taskId" => "任务",
        "statement" => "内容",
        "content" => "内容",
        _ => key,
    }
}

fn action_preview_fields(input: &Value) -> Vec<Value> {
    let preferred = [
        "name",
        "title",
        "category",
        "corePrinciple",
        "problem",
        "framework",
        "steps",
        "keyQuestions",
        "useCases",
        "statement",
        "content",
        "dueDate",
        "projectId",
        "goalId",
    ];
    let Some(object) = input.as_object() else {
        return Vec::new();
    };
    preferred
        .iter()
        .filter_map(|key| {
            object.get(*key).and_then(|value| {
                let rendered = if let Some(text) = value.as_str() {
                    text.to_string()
                } else {
                    value.to_string()
                };
                (!rendered.trim().is_empty())
                    .then(|| json!({"label": preview_label(key), "value": rendered}))
            })
        })
        .take(8)
        .collect()
}

fn agent_preview_title(tool: &str, input: &Value) -> String {
    let object_name = input
        .get("name")
        .or_else(|| input.get("title"))
        .or_else(|| input.get("statement"))
        .and_then(Value::as_str)
        .unwrap_or("新记录");
    let verb = if tool.starts_with("update") || tool == "completeTask" {
        "更新"
    } else if tool == "startTimer" {
        "开始计时"
    } else if tool == "stopTimer" {
        "停止计时"
    } else {
        "创建"
    };
    format!("准备{verb}：{object_name}")
}

fn find_agent_action_by_key(connection: &Connection, key: &str) -> Result<Option<Value>, String> {
    connection.query_row(
        "SELECT id, entity, data_json, created_at, updated_at FROM records WHERE entity='agentActions' AND archived_at IS NULL AND deleted_at IS NULL AND json_extract(data_json,'$.idempotencyKey')=?1 AND json_extract(data_json,'$.status') IN ('CONFIRM_REQUIRED','EXECUTING','SUCCESS') ORDER BY updated_at DESC LIMIT 1",
        params![key],
        |row| Ok(value_to_record(row.get(0)?, row.get(1)?, row.get(2)?, row.get(3)?, row.get(4)?)),
    ).optional().map_err(|error| error.to_string())
}

fn create_agent_action(app: AppHandle, plan: &Value, context: &Value) -> Result<Value, String> {
    let tool = plan
        .get("toolName")
        .and_then(Value::as_str)
        .ok_or("AI 没有返回可执行 Tool")?;
    let (entity, risk, requires_confirmation, action_type) =
        agent_tool_metadata(tool).ok_or_else(|| format!("不允许执行未注册 Tool：{tool}"))?;
    let connection = db(&app)?;
    let input = apply_agent_context(
        &connection,
        entity,
        action_type,
        plan.get("input").unwrap_or(&json!({})),
        context,
    )?;
    validate_agent_input(entity, action_type, &input)?;
    let idempotency_key = action_idempotency_key(tool, &input);
    if let Some(existing) = find_agent_action_by_key(&connection, &idempotency_key)? {
        return Ok(existing);
    }
    let action_id = new_id("agentActions");
    let action = json!({
        "id": action_id,
        "actionId": action_id,
        "intent": plan.get("intent").and_then(Value::as_str).unwrap_or("CREATE"),
        "toolName": tool,
        "entityType": entity,
        "input": input,
        "status": if requires_confirmation { "CONFIRM_REQUIRED" } else { "PENDING" },
        "riskLevel": risk,
        "requiresConfirmation": requires_confirmation,
        "userConfirmed": false,
        "idempotencyKey": idempotency_key,
        "previewTitle": agent_preview_title(tool, &input),
        "previewFields": action_preview_fields(&input),
        "context": context,
        "createdAt": now()
    });
    save_record(app, "agentActions".into(), action)
}

fn execute_registered_tool(app: AppHandle, action: &Value) -> Result<Value, String> {
    let tool = action
        .get("toolName")
        .and_then(Value::as_str)
        .ok_or("Action 缺少 Tool")?;
    let (entity, _, _, action_type) = agent_tool_metadata(tool).ok_or("Tool 未注册")?;
    let mut input = action.get("input").cloned().unwrap_or_else(|| json!({}));
    if let Some(object) = input.as_object_mut() {
        if let Some(action_id) = action.get("actionId").and_then(Value::as_str) {
            object.insert("agentActionId".into(), Value::String(action_id.into()));
            object.insert("evidenceLevel".into(), Value::String("AI_CONFIRMED".into()));
        }
    }
    validate_agent_input(entity, action_type, &input)?;
    match action_type {
        "CREATE" => save_record(app, entity.into(), input),
        "UPDATE" => {
            let id = input
                .get("id")
                .and_then(Value::as_str)
                .ok_or("更新操作缺少 ID")?
                .to_string();
            let existing = get_record(app.clone(), id.clone())?.ok_or("要更新的记录不存在")?;
            if existing["entity"] != entity {
                return Err("记录类型与 Tool 不匹配".into());
            }
            if let (Some(existing_object), Some(input_object)) =
                (existing.as_object(), input.as_object_mut())
            {
                for (key, value) in existing_object {
                    if !input_object.contains_key(key) {
                        input_object.insert(key.clone(), value.clone());
                    }
                }
            }
            save_record(app, entity.into(), input)
        }
        "COMPLETE" => {
            let id = input
                .get("id")
                .and_then(Value::as_str)
                .ok_or("完成任务缺少 ID")?
                .to_string();
            let mut task = get_record(app.clone(), id)?.ok_or("任务不存在")?;
            if task["entity"] != "tasks" {
                return Err("记录不是任务".into());
            }
            let object = task.as_object_mut().ok_or("任务结构无效")?;
            object.insert("status".into(), Value::String("completed".into()));
            object.insert("completedAt".into(), Value::String(now()));
            save_record(app, "tasks".into(), task)
        }
        "START_TIMER" => {
            let object = input.as_object_mut().ok_or("计时参数无效")?;
            object.insert("startAt".into(), Value::String(now()));
            object.insert("isRunning".into(), Value::Bool(true));
            object.insert("status".into(), Value::String("running".into()));
            if object
                .get("title")
                .and_then(Value::as_str)
                .unwrap_or("")
                .is_empty()
            {
                object.insert("title".into(), Value::String("专注工作".into()));
            }
            save_record(app, "timeLogs".into(), input)
        }
        "STOP_TIMER" => {
            let connection = db(&app)?;
            let running = connection.query_row("SELECT id, data_json FROM records WHERE entity='timeLogs' AND archived_at IS NULL AND deleted_at IS NULL AND json_extract(data_json,'$.isRunning')=1 LIMIT 1", [], |row| Ok((row.get::<_, String>(0)?, row.get::<_, String>(1)?))).optional().map_err(|error| error.to_string())?.ok_or("当前没有正在运行的计时")?;
            let data: Value =
                serde_json::from_str(&running.1).map_err(|error| error.to_string())?;
            let start = data
                .get("startAt")
                .and_then(Value::as_str)
                .and_then(|value| value.parse::<i64>().ok())
                .unwrap_or_else(|| now().parse::<i64>().unwrap_or_default());
            let end = now();
            let end_ms = end.parse::<i64>().unwrap_or(start);
            let minutes = ((end_ms - start) / 60000).max(1);
            stop_timer(app, running.0, end, minutes)
        }
        _ => Err("当前 Tool 动作尚未实现".into()),
    }
}

#[tauri::command]
fn execute_ai_action(app: AppHandle, action_id: String) -> Result<Value, String> {
    let connection = db(&app)?;
    let mut action = record_by_id(&connection, &action_id)?.ok_or("AI Action 不存在")?;
    if action["entity"] != "agentActions" {
        return Err("记录不是 AI Action".into());
    }
    if action["status"] == "SUCCESS" {
        return Ok(
            json!({"action": action, "record": action.get("result").cloned(), "duplicate": true}),
        );
    }
    if action["status"] == "CANCELLED" {
        return Err("该 Action 已取消".into());
    }
    let started = Instant::now();
    {
        let object = action.as_object_mut().ok_or("Action 结构无效")?;
        object.insert("status".into(), Value::String("EXECUTING".into()));
        object.insert("userConfirmed".into(), Value::Bool(true));
    }
    action = save_record(app.clone(), "agentActions".into(), action)?;
    match execute_registered_tool(app.clone(), &action) {
        Ok(record) => {
            let object = action.as_object_mut().ok_or("Action 结构无效")?;
            object.insert("status".into(), Value::String("SUCCESS".into()));
            object.insert(
                "entityId".into(),
                record.get("id").cloned().unwrap_or(Value::Null),
            );
            object.insert("result".into(), record.clone());
            object.insert("completedAt".into(), Value::String(now()));
            object.insert(
                "duration".into(),
                Value::Number((started.elapsed().as_millis() as u64).into()),
            );
            object.remove("error");
            let saved_action = save_record(app, "agentActions".into(), action)?;
            Ok(json!({"action": saved_action, "record": record, "duplicate": false}))
        }
        Err(error) => {
            let object = action.as_object_mut().ok_or("Action 结构无效")?;
            object.insert("status".into(), Value::String("FAILED".into()));
            object.insert("error".into(), Value::String(error.clone()));
            object.insert("completedAt".into(), Value::String(now()));
            object.insert(
                "duration".into(),
                Value::Number((started.elapsed().as_millis() as u64).into()),
            );
            save_record(app, "agentActions".into(), action)?;
            Err(format!("执行失败：{error}。没有写入成功。"))
        }
    }
}

#[tauri::command]
fn cancel_ai_action(app: AppHandle, action_id: String) -> Result<Value, String> {
    let connection = db(&app)?;
    let mut action = record_by_id(&connection, &action_id)?.ok_or("AI Action 不存在")?;
    if action["status"] == "SUCCESS" {
        return Err("已执行成功的 Action 不能取消".into());
    }
    let object = action.as_object_mut().ok_or("Action 结构无效")?;
    object.insert("status".into(), Value::String("CANCELLED".into()));
    object.insert("completedAt".into(), Value::String(now()));
    let saved = save_record(app, "agentActions".into(), action)?;
    Ok(json!({"action": saved}))
}

#[tauri::command]
async fn ask_chief(
    app: AppHandle,
    question: String,
    context: Option<Value>,
    history: Option<Vec<Value>>,
    schema_registry: Option<Value>,
    tool_definitions: Option<Value>,
) -> Result<Value, String> {
    tauri::async_runtime::spawn_blocking(move || {
        ask_chief_blocking(
            app,
            question,
            context,
            history,
            schema_registry,
            tool_definitions,
        )
    })
    .await
    .map_err(|error| format!("AI 后台任务失败：{error}"))?
}

fn ask_chief_blocking(
    app: AppHandle,
    question: String,
    context: Option<Value>,
    history: Option<Vec<Value>>,
    schema_registry: Option<Value>,
    tool_definitions: Option<Value>,
) -> Result<Value, String> {
    let connection = db(&app)?;
    let provider = active_provider(&connection)?;
    let key = provider_key(&provider)?;
    let model = selected_model(&connection, &provider)?;
    let page_context = context.unwrap_or_else(|| json!({}));
    let mut local_context = search_filtered(&connection, &question, &[])?;
    let mut context_ids = page_context
        .get("selectedItems")
        .and_then(Value::as_array)
        .cloned()
        .unwrap_or_default()
        .into_iter()
        .filter_map(|value| value.as_str().map(str::to_string))
        .collect::<Vec<_>>();
    for key in [
        "currentEntityId",
        "currentGoalId",
        "currentProjectId",
        "currentTaskId",
    ] {
        if let Some(id) = context_string(&page_context, key) {
            if !context_ids.contains(&id) {
                context_ids.push(id);
            }
        }
    }
    for id in context_ids {
        if let Some(record) = record_by_id(&connection, &id)? {
            if !local_context
                .iter()
                .any(|existing| existing["id"] == record["id"])
            {
                local_context.insert(0, record);
            }
        }
        for record in list_relations(app.clone(), id)? {
            if !local_context
                .iter()
                .any(|existing| existing["id"] == record["id"])
            {
                local_context.push(record);
            }
        }
    }
    for record in list_records(app.clone(), "mentalModels".into())?
        .into_iter()
        .take(12)
    {
        if !local_context
            .iter()
            .any(|existing| existing["id"] == record["id"])
        {
            local_context.push(record);
        }
    }
    let external_question = page_context.get("currentRoute").and_then(Value::as_str)
        == Some("externalIntelligence")
        || [
            "市场",
            "竞品",
            "外部信号",
            "机会",
            "趋势",
            "RedFox",
            "抖音",
            "小红书",
            "公众号",
        ]
        .iter()
        .any(|keyword| question.contains(keyword));
    if external_question {
        for entity in [
            "intelligenceBriefs",
            "signals",
            "opportunities",
            "externalSources",
        ] {
            for record in list_records(app.clone(), entity.into())?
                .into_iter()
                .take(12)
            {
                if !local_context
                    .iter()
                    .any(|existing| existing["id"] == record["id"])
                {
                    local_context.push(record);
                }
            }
        }
        local_context.push(json!({
            "id":"verified-external-items", "entity":"dataRecords", "type":"VERIFIED_EXTERNAL_TOOL_RESULT",
            "title":"外部情报已验证内容样本", "items":external_intelligence::list_items(&connection, 30)?, "evidenceLevel":"REALITY"
        }));
    }
    local_context.truncate(48);
    let context_json =
        serde_json::to_string_pretty(&local_context).map_err(|error| error.to_string())?;
    let schema_json = serde_json::to_string(&schema_registry.unwrap_or_else(|| json!([])))
        .map_err(|error| error.to_string())?;
    let tools_json = serde_json::to_string(&tool_definitions.unwrap_or_else(|| json!([])))
        .map_err(|error| error.to_string())?;
    let page_json =
        serde_json::to_string_pretty(&page_context).map_err(|error| error.to_string())?;
    let system = r#"你是 Jason OS 的 AI Agent，也是用户的首席助理。你既能分析，也能通过受控 Action System 操作 Jason OS。
必须只返回一个 JSON 对象，禁止 Markdown 代码块。格式：
{"mode":"chat|action","answer":"给用户看的简体中文","intent":"意图","toolName":"注册工具名","input":{},"missingFields":[]}
规则：
1. 用户明确要求保存、创建、更新、完成、开始或停止计时时，mode=action；不要回复“无法写入”，不要询问 Markdown/JSON/存储路径。
2. “保存到思维模型库”“沉淀为思维模型”“把刚才的模型保存”使用 createMentalModel，intent=SAVE_MENTAL_MODEL 或 CONVERT_TO_MENTAL_MODEL。
3. 从最近对话完整提取思维模型：name, category, corePrinciple, problem, framework, steps, keyQuestions, useCases, examples, outputTemplate, source, tags。多项内容可整理为换行文本。
4. 当前页面是思维模型且用户说“保存这个”时，目标就是思维模型库。
5. 创建 Project/Task/Time/Decision 时不要编造关系 ID；缺省关系由 Action System 根据当前上下文继承。
6. 需要写入时只生成 Action 预览，绝不能声称已经保存。真正写入发生在用户确认后。
7. 删除、批量修改或批量删除不要生成 Action，只说明影响并要求明确确认；当前阶段没有删除 Tool。
8. 如果缺少真正必要字段，将 mode=chat，missingFields 列出并只问最少问题。
9. 普通分析/搜索问题 mode=chat，依据本地记录回答；可以明确说明正在调用找到的思维模型。
10. 不向用户暴露 JSON、Schema、数据库、SQL、内部 API 或 Tool 技术细节。
11. 当当前页面上下文 analysisMode=timeline_readonly 时，只能进行基于证据的只读分析，mode 必须为 chat，禁止生成 Action。
12. External Intelligence 分析必须区分事实、确定性计算、AI 推断、数据缺口和替代解释；不得把爆款等同于需求，不得把转载等同于独立信号。
13. 创建情报源、机会或由信号生成决策时必须生成 Action 预览；不得自动创建 Project、增加预算、采购或修改财务。
14. 外部指标只能引用 VERIFIED_EXTERNAL_TOOL_RESULT、Signal 或真实本地记录；缺失时必须说明数据不足。"#;
    let mut messages = Vec::new();
    let recent_history = history.unwrap_or_default();
    let skip = recent_history.len().saturating_sub(10);
    for message in recent_history.into_iter().skip(skip) {
        if let (Some(role), Some(content)) = (message["role"].as_str(), message["content"].as_str())
        {
            if ["user", "assistant"].contains(&role) {
                messages.push(json!({"role": role, "content": content}));
            }
        }
    }
    messages.push(json!({"role":"user","content":format!("当前页面上下文：\n{page_json}\n\nJason OS Schema Registry：\n{schema_json}\n\n允许使用的 Tool Registry：\n{tools_json}\n\n相关本地记录与可调用思维模型：\n{context_json}\n\n当前用户请求：\n{question}")}));
    let (response_json, _) = call_provider(&provider, &key, &model, system, &messages, 2800)?;
    let raw = provider_response_text(&provider, &response_json);
    if raw.trim().is_empty() {
        return Err(format!("{} 未返回文本内容", provider_label(&provider)));
    }
    let parsed = extract_json_value(&raw);
    let mut answer = parsed
        .as_ref()
        .and_then(|value| value.get("answer"))
        .and_then(Value::as_str)
        .unwrap_or(&raw)
        .trim()
        .to_string();
    let mut action: Option<Value> = None;
    if !timeline_analysis_is_read_only(&page_context)
        && parsed
            .as_ref()
            .and_then(|value| value.get("mode"))
            .and_then(Value::as_str)
            == Some("action")
    {
        let plan = parsed.as_ref().unwrap();
        match create_agent_action(app.clone(), plan, &page_context) {
            Ok(saved_action) => {
                let status = saved_action
                    .get("status")
                    .and_then(Value::as_str)
                    .unwrap_or("");
                answer = if status == "SUCCESS" {
                    "相同操作已经执行成功，为避免重复，没有再次创建。".into()
                } else {
                    format!(
                        "{}。请确认后执行。",
                        saved_action
                            .get("previewTitle")
                            .and_then(Value::as_str)
                            .unwrap_or("我已整理好本次操作")
                    )
                };
                action = Some(saved_action);
            }
            Err(error) => {
                answer = format!("我理解了你的操作意图，但当前还不能安全执行：{error}");
            }
        }
    }
    let run = json!({
        "id": new_id("agentRuns"), "agentType": format!("{}-agent-planner", provider), "provider": provider,
        "input": question, "context": local_context, "pageContext": page_context, "output": answer,
        "actionId": action.as_ref().and_then(|value| value.get("actionId")).cloned(),
        "status": "completed", "model": model, "startedAt": now(), "completedAt": now()
    });
    let saved_run = save_record(app, "agentRuns".into(), run)?;
    Ok(
        json!({"answer": saved_run["output"], "context": saved_run["context"], "action": action, "agentRun": saved_run}),
    )
}

pub fn run() {
    tauri::Builder::default()
        .plugin(tauri_plugin_log::Builder::default().build())
        .setup(|app| {
            db(&app.handle()).map_err(|e| std::io::Error::new(std::io::ErrorKind::Other, e))?;
            Ok(())
        })
        .invoke_handler(tauri::generate_handler![
            initialize_database,
            check_relationship_integrity,
            capture_link,
            get_capture_provider_config,
            configure_capture_provider,
            test_capture_provider,
            list_external_items,
            cleanup_external_cache,
            list_records,
            get_record,
            save_record,
            delete_record,
            archive_record,
            restore_record,
            list_archived,
            search_records,
            search_records_filtered,
            list_relations,
            add_relation,
            stop_timer,
            export_data,
            create_backup,
            list_backups,
            restore_backup,
            open_external,
            get_ai_config,
            configure_ai_provider,
            test_ai_provider,
            get_hackstart_config,
            configure_hackstart,
            ask_chief,
            execute_ai_action,
            cancel_ai_action
        ])
        .run(tauri::generate_context!())
        .expect("error while running Jason OS");
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn external_intelligence_entities_and_agent_tools_are_registered() {
        for entity in [
            "externalSources",
            "signals",
            "opportunities",
            "intelligenceBriefs",
        ] {
            assert!(is_entity(entity));
        }
        assert_eq!(
            agent_tool_metadata("createExternalSource").unwrap().0,
            "externalSources"
        );
        assert_eq!(
            agent_tool_metadata("createOpportunity").unwrap().0,
            "opportunities"
        );
    }

    #[test]
    fn capture_provider_config_never_contains_the_api_key() {
        let config = capture_provider_config(true);
        let serialized = serde_json::to_string(&config).unwrap();
        assert!(config["providers"][0]["configured"].as_bool().unwrap());
        assert!(!serialized.contains("apiKey"));
        assert!(!serialized.contains("REDFOX_API_KEY"));
        assert!(!serialized.contains("secret"));
    }

    #[test]
    fn agent_registry_allows_mental_model_writes_but_not_delete_tools() {
        assert_eq!(
            agent_tool_metadata("createMentalModel").unwrap().0,
            "mentalModels"
        );
        assert!(agent_tool_metadata("deleteProject").is_none());
    }

    #[test]
    fn agent_structured_output_accepts_json_inside_code_fences() {
        let parsed = extract_json_value(
            "```json\n{\"mode\":\"action\",\"toolName\":\"createMentalModel\"}\n```",
        )
        .unwrap();
        assert_eq!(parsed["toolName"], "createMentalModel");
    }

    #[test]
    fn agent_write_validation_requires_a_mental_model_name() {
        assert!(
            validate_agent_input("mentalModels", "CREATE", &json!({"name":"芒格式决策树"})).is_ok()
        );
        assert!(validate_agent_input(
            "mentalModels",
            "CREATE",
            &json!({"corePrinciple":"逆向思考"})
        )
        .is_err());
    }

    #[test]
    fn repeated_agent_inputs_have_the_same_idempotency_key() {
        let input = json!({"name":"芒格式决策树","category":"决策模型"});
        assert_eq!(
            action_idempotency_key("createMentalModel", &input),
            action_idempotency_key("createMentalModel", &input)
        );
    }

    #[test]
    fn timeline_metadata_separates_planned_and_actual_time() {
        let planned = apply_timeline_metadata(
            "tasks",
            &json!({"status":"todo","dueDate":"2026-08-12"}),
            "2026-08-10T08:00:00-07:00",
            "2026-08-11T08:00:00-07:00",
        );
        assert_eq!(planned["occurredAt"], "2026-08-12");
        assert_eq!(planned["timeMeaning"], "planned");
        assert_eq!(planned["timeZone"], "local");
        let actual = apply_timeline_metadata(
            "tasks",
            &json!({"status":"completed","completedAt":"2026-08-11T09:30:00-07:00"}),
            "2026-08-10T08:00:00-07:00",
            "2026-08-11T09:30:00-07:00",
        );
        assert_eq!(actual["occurredAt"], "2026-08-11T09:30:00-07:00");
        assert_eq!(actual["timeMeaning"], "actual");
        assert_eq!(actual["timeZone"], "offset");
    }

    #[test]
    fn timeline_metadata_keeps_legacy_reviews_on_created_time() {
        let review = apply_timeline_metadata(
            "reviews",
            &json!({"title":"Review"}),
            "2026-08-01T10:00:00-07:00",
            "2026-08-11T10:00:00-07:00",
        );
        assert_eq!(review["occurredAt"], "2026-08-01T10:00:00-07:00");
        assert_eq!(review["timeMeaning"], "recorded");
    }

    #[test]
    fn timeline_change_events_are_idempotent() {
        let connection = relationship_test_db();
        put(
            &connection,
            "tasks",
            "task-1",
            json!({"title":"Task","status":"todo"}),
        );
        let before = json!({"title":"Task","status":"todo"});
        let after = json!({"title":"Task","status":"completed","taskId":"task-1"});
        write_timeline_change_events(&connection, "tasks", "task-1", Some(&before), &after)
            .unwrap();
        write_timeline_change_events(&connection, "tasks", "task-1", Some(&before), &after)
            .unwrap();
        let count: i64 = connection
            .query_row(
                "SELECT COUNT(*) FROM records WHERE entity='timelineEvents'",
                [],
                |row| row.get(0),
            )
            .unwrap();
        assert_eq!(count, 1);
    }

    #[test]
    fn external_source_capacity_limits_new_and_reactivated_sources() {
        let connection = relationship_test_db();
        for index in 0..20 {
            put(
                &connection,
                "externalSources",
                &format!("source-{index}"),
                json!({"name":format!("Source {index}"),"status":"active"}),
            );
        }
        assert!(ensure_external_source_capacity(
            &connection,
            "externalSources",
            "source-new",
            &json!({"name":"New","status":"active"})
        )
        .is_err());
        assert!(ensure_external_source_capacity(
            &connection,
            "externalSources",
            "source-0",
            &json!({"name":"Existing","status":"active"})
        )
        .is_ok());
        put(
            &connection,
            "externalSources",
            "source-paused",
            json!({"name":"Paused","status":"paused"}),
        );
        assert!(ensure_external_source_capacity(
            &connection,
            "externalSources",
            "source-paused",
            &json!({"name":"Paused","status":"active"})
        )
        .is_err());
    }

    #[test]
    fn external_decisions_validate_signal_and_opportunity_relations() {
        let connection = relationship_test_db();
        put(
            &connection,
            "signals",
            "signal-1",
            json!({"title":"Signal","status":"observing"}),
        );
        put(
            &connection,
            "opportunities",
            "opportunity-1",
            json!({"title":"Opportunity","status":"draft"}),
        );
        let decision = normalize_record(
            &connection,
            "decisions",
            &json!({"title":"Decision","signalIds":["signal-1"],"opportunityId":"opportunity-1"}),
            true,
        )
        .unwrap();
        assert_eq!(decision["signalIds"], json!(["signal-1"]));
        assert_eq!(decision["opportunityId"], "opportunity-1");
        assert!(normalize_record(
            &connection,
            "decisions",
            &json!({"title":"Bad decision","signalIds":["missing"]}),
            true,
        )
        .is_err());
    }

    #[test]
    fn causal_relation_ids_must_reference_the_expected_entity() {
        let connection = relationship_test_db();
        put(
            &connection,
            "decisions",
            "decision-1",
            json!({"title":"Decision"}),
        );
        let task = normalize_record(
            &connection,
            "tasks",
            &json!({"title":"Task","decisionId":"decision-1"}),
            true,
        )
        .unwrap();
        assert_eq!(task["decisionId"], "decision-1");
        assert!(normalize_record(
            &connection,
            "tasks",
            &json!({"title":"Task","decisionId":"missing"}),
            true
        )
        .is_err());
    }

    #[test]
    fn causal_context_is_inherited_and_conflicts_are_rejected() {
        let connection = relationship_test_db();
        put(&connection, "goals", "goal-1", json!({"title":"Goal 1"}));
        put(&connection, "goals", "goal-2", json!({"title":"Goal 2"}));
        put(
            &connection,
            "projects",
            "project-1",
            json!({"title":"Project 1","goalId":"goal-1"}),
        );
        put(
            &connection,
            "projects",
            "project-2",
            json!({"title":"Project 2","goalId":"goal-2"}),
        );
        put(
            &connection,
            "decisions",
            "decision-1",
            json!({"title":"Decision","projectId":"project-1"}),
        );
        let task = normalize_record(
            &connection,
            "tasks",
            &json!({"title":"Task","decisionId":"decision-1"}),
            true,
        )
        .unwrap();
        assert_eq!(task["projectId"], "project-1");
        assert_eq!(task["goalId"], "goal-1");
        assert!(normalize_record(
            &connection,
            "tasks",
            &json!({"title":"Wrong Task","projectId":"project-2","decisionId":"decision-1"}),
            true,
        )
        .is_err());

        put(
            &connection,
            "tasks",
            "task-1",
            json!({"title":"Task","projectId":"project-1"}),
        );
        put(
            &connection,
            "results",
            "result-1",
            json!({"title":"Result","taskId":"task-1"}),
        );
        put(
            &connection,
            "reviews",
            "review-1",
            json!({"title":"Review","resultId":"result-1"}),
        );
        let review = record_by_id(&connection, "review-1").unwrap().unwrap();
        assert_eq!(review["taskId"], "task-1");
        assert_eq!(review["projectId"], "project-1");
        assert_eq!(review["goalId"], "goal-1");

        let insight = normalize_record(
            &connection,
            "insights",
            &json!({"statement":"Insight","reviewId":"review-1"}),
            true,
        )
        .unwrap();
        assert_eq!(insight["taskId"], "task-1");
        assert_eq!(insight["projectId"], "project-1");
        assert_eq!(insight["goalId"], "goal-1");
    }

    #[test]
    fn timeline_analysis_mode_is_read_only() {
        assert!(timeline_analysis_is_read_only(
            &json!({"analysisMode":"timeline_readonly"})
        ));
        assert!(!timeline_analysis_is_read_only(
            &json!({"currentRoute":"timeline"})
        ));
    }

    #[test]
    fn external_links_are_limited_to_web_urls() {
        assert!(is_external_url("https://openai.com/news"));
        assert!(is_external_url("http://example.com"));
        assert!(!is_external_url("file:///tmp/private"));
        assert!(!is_external_url("javascript:alert(1)"));
    }

    #[test]
    fn only_declared_entities_are_accepted() {
        assert!(is_entity("goals"));
        assert!(!is_entity("drop table records"));
    }
    #[test]
    fn a_title_can_be_derived_from_a_record() {
        let (title, _) = title_body(&json!({"name": "First principle"}));
        assert_eq!(title, "First principle");
    }
    #[test]
    fn generated_ids_do_not_collide_within_the_same_millisecond() {
        assert_ne!(new_id("goals"), new_id("goals"));
    }
    #[test]
    fn current_provider_model_catalog_is_explicit() {
        assert_eq!(
            provider_models("deepseek"),
            &["deepseek-v4-pro", "deepseek-v4-flash"]
        );
        assert_eq!(provider_models("minimax"), &["MiniMax-M3"]);
        assert_eq!(
            provider_base_url("minimax"),
            "https://api.minimaxi.com/anthropic/v1/messages"
        );
        assert_eq!(provider_models("volc-agent-plan"), &["kimi-k3"]);
        assert_eq!(
            provider_base_url("volc-agent-plan"),
            "https://ark.cn-beijing.volces.com/api/plan/v3/responses"
        );
    }
    #[test]
    fn relation_fields_are_materialized() {
        let connection = Connection::open_in_memory().unwrap();
        connection.execute_batch("CREATE TABLE records(id TEXT PRIMARY KEY, data_json TEXT, deleted_at TEXT); CREATE TABLE relations(id TEXT PRIMARY KEY, from_id TEXT, to_id TEXT, relation_type TEXT, created_at TEXT, UNIQUE(from_id,to_id,relation_type));").unwrap();
        connection
            .execute(
                "INSERT INTO records(id,data_json) VALUES ('project-1','{}'),('task-1','{}')",
                [],
            )
            .unwrap();
        sync_relations(&connection, "task-1", &json!({"projectId":"project-1"})).unwrap();
        let count: i64 = connection.query_row("SELECT COUNT(*) FROM relations WHERE from_id='task-1' AND to_id='project-1' AND relation_type='field:projectId'", [], |row| row.get(0)).unwrap();
        assert_eq!(count, 1);
    }

    fn relationship_test_db() -> Connection {
        let connection = Connection::open_in_memory().unwrap();
        connection.execute_batch("PRAGMA foreign_keys=ON; CREATE TABLE records(id TEXT PRIMARY KEY,entity TEXT NOT NULL,data_json TEXT NOT NULL,title TEXT NOT NULL DEFAULT '',body TEXT NOT NULL DEFAULT '',created_at TEXT NOT NULL,updated_at TEXT NOT NULL,archived_at TEXT,deleted_at TEXT); CREATE VIRTUAL TABLE records_fts USING fts5(id UNINDEXED,entity UNINDEXED,title,body); CREATE TABLE relations(id TEXT PRIMARY KEY,from_id TEXT NOT NULL,to_id TEXT NOT NULL,relation_type TEXT NOT NULL,created_at TEXT NOT NULL,UNIQUE(from_id,to_id,relation_type));").unwrap();
        connection
    }

    fn put(connection: &Connection, entity: &str, id: &str, data: Value) {
        let normalized = normalize_record(connection, entity, &data, true).unwrap();
        write_record(connection, entity, id, &normalized, None).unwrap();
        sync_relations(connection, id, &normalized).unwrap();
    }

    fn seed_work_chain(connection: &Connection) {
        put(connection, "goals", "goal-a", json!({"title":"Goal A"}));
        put(connection, "goals", "goal-b", json!({"title":"Goal B"}));
        put(
            connection,
            "projects",
            "project-b",
            json!({"title":"Project B","goalId":"goal-a"}),
        );
        put(
            connection,
            "tasks",
            "task-c",
            json!({"title":"Task C","projectId":"project-b"}),
        );
    }

    #[test]
    fn relationship_test_1_goal_project_task_chain() {
        let connection = relationship_test_db();
        seed_work_chain(&connection);
        let task = record_by_id(&connection, "task-c").unwrap().unwrap();
        assert_eq!(task["projectId"], "project-b");
        assert_eq!(task["goalId"], "goal-a");
    }

    #[test]
    fn relationship_test_2_project_created_from_goal() {
        let connection = relationship_test_db();
        put(&connection, "goals", "goal-a", json!({"title":"A"}));
        put(
            &connection,
            "projects",
            "project-b",
            json!({"title":"B","goalId":"goal-a"}),
        );
        assert_eq!(
            record_by_id(&connection, "project-b").unwrap().unwrap()["goalId"],
            "goal-a"
        );
    }

    #[test]
    fn relationship_test_3_task_created_from_project_inherits_goal() {
        let connection = relationship_test_db();
        seed_work_chain(&connection);
        let task = record_by_id(&connection, "task-c").unwrap().unwrap();
        assert_eq!(task["goalId"], "goal-a");
    }

    #[test]
    fn relationship_test_4_timer_from_task_inherits_project_and_goal() {
        let connection = relationship_test_db();
        seed_work_chain(&connection);
        let timer = normalize_record(
            &connection,
            "timeLogs",
            &json!({"taskId":"task-c","isRunning":true}),
            true,
        )
        .unwrap();
        assert_eq!(timer["taskId"], "task-c");
        assert_eq!(timer["projectId"], "project-b");
        assert_eq!(timer["goalId"], "goal-a");
    }

    #[test]
    fn relationship_test_5_timer_from_project_inherits_goal() {
        let connection = relationship_test_db();
        seed_work_chain(&connection);
        let timer = normalize_record(
            &connection,
            "timeLogs",
            &json!({"projectId":"project-b"}),
            true,
        )
        .unwrap();
        assert_eq!(timer["projectId"], "project-b");
        assert_eq!(timer["goalId"], "goal-a");
    }

    #[test]
    fn relationship_test_6_timer_from_goal_is_valid() {
        let connection = relationship_test_db();
        put(&connection, "goals", "goal-a", json!({"title":"A"}));
        let timer =
            normalize_record(&connection, "timeLogs", &json!({"goalId":"goal-a"}), true).unwrap();
        assert_eq!(timer["goalId"], "goal-a");
    }

    #[test]
    fn relationship_test_7_independent_timer_is_allowed() {
        let connection = relationship_test_db();
        let timer = normalize_record(
            &connection,
            "timeLogs",
            &json!({"title":"Independent"}),
            true,
        )
        .unwrap();
        assert!(timer.get("taskId").is_none());
        assert!(timer.get("projectId").is_none());
        assert!(timer.get("goalId").is_none());
    }

    #[test]
    fn relationship_test_8_project_goal_change_cascades_to_tasks() {
        let connection = relationship_test_db();
        seed_work_chain(&connection);
        put(
            &connection,
            "projects",
            "project-b",
            json!({"title":"B","goalId":"goal-b"}),
        );
        cascade_context(&connection, "projects", "project-b").unwrap();
        assert_eq!(
            record_by_id(&connection, "task-c").unwrap().unwrap()["goalId"],
            "goal-b"
        );
    }

    #[test]
    fn relationship_test_9_goal_aggregates_project_task_and_time() {
        let connection = relationship_test_db();
        seed_work_chain(&connection);
        put(
            &connection,
            "timeLogs",
            "time-1",
            json!({"taskId":"task-c","durationMinutes":30}),
        );
        let records = all_records(&connection).unwrap();
        assert_eq!(
            records
                .iter()
                .filter(|record| record["goalId"] == "goal-a")
                .count(),
            3
        );
    }

    #[test]
    fn relationship_test_10_project_aggregates_tasks_and_time() {
        let connection = relationship_test_db();
        seed_work_chain(&connection);
        put(
            &connection,
            "timeLogs",
            "time-1",
            json!({"taskId":"task-c","durationMinutes":30}),
        );
        let records = all_records(&connection).unwrap();
        assert_eq!(
            records
                .iter()
                .filter(|record| record["projectId"] == "project-b")
                .count(),
            2
        );
    }

    #[test]
    fn relationship_test_11_task_aggregates_time_and_result() {
        let connection = relationship_test_db();
        seed_work_chain(&connection);
        put(
            &connection,
            "timeLogs",
            "time-1",
            json!({"taskId":"task-c"}),
        );
        put(
            &connection,
            "results",
            "result-1",
            json!({"taskId":"task-c","title":"Result"}),
        );
        let records = all_records(&connection).unwrap();
        assert_eq!(
            records
                .iter()
                .filter(|record| record["taskId"] == "task-c")
                .count(),
            2
        );
    }

    #[test]
    fn relationship_test_12_mismatched_task_goal_is_auto_corrected() {
        let connection = relationship_test_db();
        seed_work_chain(&connection);
        let task = normalize_record(
            &connection,
            "tasks",
            &json!({"projectId":"project-b","goalId":"goal-b"}),
            true,
        )
        .unwrap();
        assert_eq!(task["goalId"], "goal-a");
    }

    #[test]
    fn capture_platforms_are_detected() {
        assert_eq!(
            capture_platform("https://mp.weixin.qq.com/s/a"),
            "微信公众号"
        );
        assert_eq!(capture_platform("https://www.douyin.com/video/1"), "抖音");
        assert_eq!(
            capture_platform("https://www.xiaohongshu.com/explore/1"),
            "小红书"
        );
        assert_eq!(capture_platform("https://x.com/openai/status/1"), "X");
        assert_eq!(
            capture_platform("https://www.instagram.com/p/1"),
            "Instagram"
        );
        assert_eq!(
            capture_platform("https://www.reddit.com/r/rust/1"),
            "Reddit"
        );
        assert_eq!(
            capture_platform("https://www.facebook.com/watch/1"),
            "Facebook"
        );
    }
}
