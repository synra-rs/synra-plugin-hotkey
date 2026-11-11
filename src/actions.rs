//! Fixed hotkey delivery and the desktop receiver event.

use serde_json::Value;
use std::collections::HashMap;
use std::sync::{Mutex, OnceLock};
use synra_plugin_sdk::{Error, Outcome, host};

pub const EXECUTE: &str = "hotkey.execute";
pub const RESPONSE_POLL: &str = "hotkey.response-poll";
pub const EVENT_EXECUTE: &str = "_plugin.@synra-plugin/hotkey.execute";
pub const EVENT_RESPONSE: &str = "_plugin.@synra-plugin/hotkey.response";

const MAX_ARGS: usize = 4096;
const MAX_REQUEST_ID: usize = 128;
const MAX_SHORTCUT_ID: usize = 64;
const MAX_PLATFORM: usize = 16;
const MAX_RESPONSES: usize = 64;

static RESPONSES: OnceLock<Mutex<HashMap<String, Value>>> = OnceLock::new();

fn responses() -> &'static Mutex<HashMap<String, Value>> {
    RESPONSES.get_or_init(|| Mutex::new(HashMap::new()))
}

/// Return the only platform-specific chord that can be emitted for a built-in shortcut id.
pub fn chord_for(shortcut_id: &str, platform: &str) -> Option<&'static str> {
    Some(match platform {
        // Windows SendKeys uses ^ for Ctrl. `{LWIN}` is the SendKeys spelling for the Windows modifier on hosts that
        // expose it; the host may still refuse it when the underlying SendKeys implementation lacks Win-key support.
        "windows" => match shortcut_id {
            "copy" => "^c",
            "cut" => "^x",
            "paste" => "^v",
            "undo" => "^z",
            "redo" => "^y",
            "select-all" => "^a",
            "save" => "^s",
            "find" => "^f",
            "open-folder" => "{LWIN}e",
            _ => return None,
        },
        "linux" => match shortcut_id {
            "copy" => "ctrl+c",
            "cut" => "ctrl+x",
            "paste" => "ctrl+v",
            "undo" => "ctrl+z",
            "redo" => "ctrl+y",
            "select-all" => "ctrl+a",
            "save" => "ctrl+s",
            "find" => "ctrl+f",
            "open-folder" => "super+e",
            _ => return None,
        },
        "macos" => match shortcut_id {
            "copy" => "cmd+c",
            "cut" => "cmd+x",
            "paste" => "cmd+v",
            "undo" => "cmd+z",
            "redo" => "cmd+y",
            "select-all" => "cmd+a",
            "save" => "cmd+s",
            "find" => "cmd+f",
            "open-folder" => "cmd+shift+g",
            _ => return None,
        },
        "desktop" => match shortcut_id {
            "copy" => "ctrl+c",
            "cut" => "ctrl+x",
            "paste" => "ctrl+v",
            "undo" => "ctrl+z",
            "redo" => "ctrl+y",
            "select-all" => "ctrl+a",
            "save" => "ctrl+s",
            "find" => "ctrl+f",
            "open-folder" => "win+e",
            _ => return None,
        },
        _ => return None,
    })
}

pub(crate) fn declare() -> Outcome {
    for (name, schema) in [
        (
            EXECUTE,
            r#"{"type":"object","properties":{"target":{"type":"string"},"requestId":{"type":"string"},"shortcutId":{"type":"string"},"platform":{"type":"string"}},"required":["target","requestId","shortcutId","platform"],"additionalProperties":false}"#,
        ),
        (
            RESPONSE_POLL,
            r#"{"type":"object","properties":{"requestId":{"type":"string"}},"required":["requestId"],"additionalProperties":false}"#,
        ),
    ] {
        if let Outcome::Err(error) = host::action_declare(name, schema) {
            return Outcome::Err(error);
        }
    }
    Outcome::Ok(String::new())
}

pub(crate) fn invoke(name: &str, args: &str) -> Outcome {
    if args.len() > MAX_ARGS {
        return refused("badArgument", "hotkey arguments exceed 4 KiB");
    }
    let value = match serde_json::from_str::<Value>(args) {
        Ok(Value::Object(value)) => value,
        Ok(_) => return refused("badArgument", "hotkey arguments must be a JSON object"),
        Err(_) => return refused("badArgument", "hotkey arguments are not valid JSON"),
    };
    match name {
        EXECUTE => execute(&value),
        RESPONSE_POLL => poll_response(&value),
        _ => refused("actionNotFound", "unknown Hotkey action"),
    }
}

fn execute(value: &serde_json::Map<String, Value>) -> Outcome {
    let target = match target(value) {
        Ok(value) => value,
        Err(error) => return error,
    };
    let request_id = match request_id(value) {
        Ok(value) => value,
        Err(error) => return error,
    };
    let shortcut_id = match required_string(value, "shortcutId", MAX_SHORTCUT_ID) {
        Ok(value) => value,
        Err(error) => return error,
    };
    let platform = match platform(value) {
        Ok(value) => value,
        Err(error) => return error,
    };
    if chord_for(shortcut_id, platform).is_none() {
        return refused("badArgument", "shortcutId is not a built-in shortcut");
    }
    let payload = serde_json::json!({
        "requestId": request_id,
        "shortcutId": shortcut_id,
        "platform": platform,
    });
    match host::device_send(target, EVENT_EXECUTE, &payload.to_string()) {
        Outcome::Ok(_) => Outcome::Ok(
            serde_json::json!({ "status": "queued", "requestId": request_id }).to_string(),
        ),
        Outcome::Err(error) => Outcome::Err(error),
    }
}

pub(crate) fn on_event(event_json: &str) -> Outcome {
    let event = match serde_json::from_str::<Value>(event_json) {
        Ok(Value::Object(value)) => value,
        _ => return refused("badArgument", "event context is not valid JSON"),
    };
    let name = match event.get("event").and_then(Value::as_str) {
        Some(value) => value,
        None => return refused("badArgument", "event context has no event name"),
    };
    let payload = match event.get("payload") {
        Some(Value::Object(value)) => value,
        _ => return refused("badArgument", "event context has no object payload"),
    };
    let from = match event.get("from").and_then(Value::as_str) {
        Some(value) if is_device_id(value) => value,
        _ => return refused("badArgument", "event context has no valid sender device"),
    };

    if name == EVENT_RESPONSE {
        let request_id = match request_id(payload) {
            Ok(value) => value,
            Err(error) => return error,
        };
        let Some(status) = payload.get("status").and_then(Value::as_str) else {
            return refused("badArgument", "event response has no status");
        };
        if !matches!(status, "pending" | "ok" | "error") {
            return refused("badArgument", "event response status is invalid");
        }
        let Ok(mut stored) = responses().lock() else {
            return refused("internal", "event response storage is unavailable");
        };
        if stored.len() >= MAX_RESPONSES {
            stored.clear();
        }
        stored.insert(request_id.to_owned(), Value::Object(payload.clone()));
        return Outcome::Ok(String::new());
    }

    if name != EVENT_EXECUTE {
        return refused("notImplemented", "this plugin does not handle that event");
    }
    let request_id = match request_id(payload) {
        Ok(value) => value,
        Err(error) => return error,
    };
    let shortcut_id = match required_string(payload, "shortcutId", MAX_SHORTCUT_ID) {
        Ok(value) => value,
        Err(error) => return reply(from, request_id, error),
    };
    let platform = match platform(payload) {
        Ok(value) => value,
        Err(error) => return reply(from, request_id, error),
    };
    let Some(chord) = chord_for(shortcut_id, platform) else {
        return reply(
            from,
            request_id,
            refused("badArgument", "shortcutId is not a built-in shortcut"),
        );
    };
    reply(from, request_id, host::platform_simulate_keys(chord))
}

fn poll_response(value: &serde_json::Map<String, Value>) -> Outcome {
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

fn reply(from: &str, request_id: &str, outcome: Outcome) -> Outcome {
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
    match host::device_send(from, EVENT_RESPONSE, &payload.to_string()) {
        Outcome::Ok(_) => Outcome::Ok(String::new()),
        Outcome::Err(error) => Outcome::Err(error),
    }
}

fn target(value: &serde_json::Map<String, Value>) -> Result<&str, Outcome> {
    let target = required_string(value, "target", 64)?;
    if !is_device_id(target) {
        return Err(refused(
            "badArgument",
            "target must be a canonical device id",
        ));
    }
    Ok(target)
}

fn request_id(value: &serde_json::Map<String, Value>) -> Result<&str, Outcome> {
    required_string(value, "requestId", MAX_REQUEST_ID)
}

fn platform(value: &serde_json::Map<String, Value>) -> Result<&str, Outcome> {
    let platform = required_string(value, "platform", MAX_PLATFORM)?;
    if !matches!(platform, "windows" | "linux" | "macos" | "desktop") {
        return Err(refused("badArgument", "platform is not supported"));
    }
    Ok(platform)
}

fn required_string<'a>(
    value: &'a serde_json::Map<String, Value>,
    key: &str,
    max: usize,
) -> Result<&'a str, Outcome> {
    let Some(value) = value.get(key).and_then(Value::as_str) else {
        return Err(refused(
            "badArgument",
            "required hotkey argument is missing",
        ));
    };
    if value.is_empty() || value.len() > max {
        return Err(refused(
            "badArgument",
            "hotkey argument has an invalid length",
        ));
    }
    Ok(value)
}

fn is_device_id(value: &str) -> bool {
    value.len() == 36
        && value.as_bytes().iter().enumerate().all(|(index, byte)| {
            if matches!(index, 8 | 13 | 18 | 23) {
                *byte == b'-'
            } else {
                byte.is_ascii_hexdigit()
            }
        })
}

fn refused(code: &str, message: &str) -> Outcome {
    Outcome::Err(Error {
        code: code.to_owned(),
        message: message.to_owned(),
        details: None,
    })
}

#[cfg(test)]
mod tests {
    use super::chord_for;

    #[test]
    fn built_in_shortcuts_have_fixed_chords() {
        assert_eq!(chord_for("copy", "windows"), Some("^c"));
        assert_eq!(chord_for("paste", "linux"), Some("ctrl+v"));
        assert_eq!(chord_for("open-folder", "desktop"), Some("win+e"));
    }

    #[test]
    fn unknown_shortcuts_are_refused() {
        assert_eq!(chord_for("ctrl-alt-delete", "windows"), None);
    }
}
