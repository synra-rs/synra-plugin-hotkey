use serde_json::Value;
use std::collections::HashMap;
use std::sync::{Mutex, OnceLock};
use synra_plugin_sdk::{Outcome, host};

use super::validation::{refused, request_id};
use super::{EVENT_RESPONSE, log_info};

const MAX_RESPONSES: usize = 64;

static RESPONSES: OnceLock<Mutex<HashMap<String, Value>>> = OnceLock::new();

fn responses() -> &'static Mutex<HashMap<String, Value>> {
    RESPONSES.get_or_init(|| Mutex::new(HashMap::new()))
}

pub(super) fn store(request_id: &str, payload: Value) -> Result<(), Outcome> {
    let Ok(mut stored) = responses().lock() else {
        return Err(refused("internal", "event response storage is unavailable"));
    };
    if stored.len() >= MAX_RESPONSES {
        stored.clear();
    }
    stored.insert(request_id.to_owned(), payload);
    Ok(())
}

pub(super) fn poll(value: &serde_json::Map<String, Value>) -> Outcome {
    let request_id = match request_id(value) {
        Ok(value) => value,
        Err(error) => return error,
    };
    let Ok(mut stored) = responses().lock() else {
        return refused("internal", "event response storage is unavailable");
    };
    Outcome::Ok(
        stored
            .remove(request_id)
            .unwrap_or_else(|| serde_json::json!({ "status": "pending", "requestId": request_id }))
            .to_string(),
    )
}

pub(super) fn reply(from: &str, request_id: &str, outcome: Outcome) -> Outcome {
    let payload = match outcome {
        Outcome::Ok(value) => {
            let value = serde_json::from_str::<Value>(&value).unwrap_or(Value::String(value));
            serde_json::json!({
                "requestId": request_id,
                "status": "ok",
                "value": value,
            })
        }
        Outcome::Err(error) => serde_json::json!({
            "requestId": request_id,
            "status": "error",
            "code": error.code,
            "message": error.message,
        }),
    };
    let status = payload
        .get("status")
        .and_then(Value::as_str)
        .unwrap_or("unknown");
    log_info(format!(
        "hotkey response sending requestId={request_id} status={status} target={from}"
    ));
    match host::device_send(from, EVENT_RESPONSE, &payload.to_string()) {
        Outcome::Ok(_) => {
            log_info(format!(
                "hotkey response sent requestId={request_id} status={status} target={from}"
            ));
            Outcome::Ok(String::new())
        }
        Outcome::Err(error) => {
            log_info(format!(
                "hotkey response send failed requestId={request_id} code={} message={}",
                error.code, error.message
            ));
            Outcome::Err(error)
        }
    }
}
