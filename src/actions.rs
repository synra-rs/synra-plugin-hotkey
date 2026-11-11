//! Fixed hotkey delivery and the desktop receiver event.

use serde_json::Value;
use synra_plugin_sdk::{LogLevel, Outcome, host};

#[path = "responses.rs"]
mod responses;
#[path = "validation.rs"]
mod validation;

use validation::{is_device_id, platform, refused, request_id, required_string, target};

pub const EXECUTE: &str = "hotkey.execute";
pub const RESPONSE_POLL: &str = "hotkey.response-poll";
pub const EVENT_EXECUTE: &str = "_plugin.@synra-plugin/hotkey.execute";
pub const EVENT_RESPONSE: &str = "_plugin.@synra-plugin/hotkey.response";

const MAX_ARGS: usize = 4096;
const MAX_REQUEST_ID: usize = 128;
const MAX_SHORTCUT_ID: usize = 64;
const MAX_PLATFORM: usize = 16;
fn log_info(message: impl Into<String>) {
    let message = message.into();
    host::log(LogLevel::Info, &message, None);
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
        // Discovery may only report the generic desktop kind. Synra's current desktop receiver is Windows, so use the
        // Windows SendKeys spelling for this fallback rather than sending the literal `ctrl+c` text.
        "desktop" => match shortcut_id {
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
        RESPONSE_POLL => responses::poll(&value),
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
        Outcome::Ok(_) => {
            log_info(format!(
                "hotkey execute queued requestId={request_id} shortcutId={shortcut_id} platform={platform} target={target}"
            ));
            Outcome::Ok(
                serde_json::json!({ "status": "queued", "requestId": request_id }).to_string(),
            )
        }
        Outcome::Err(error) => {
            log_info(format!(
                "hotkey execute queue failed requestId={request_id} code={} message={}",
                error.code, error.message
            ));
            Outcome::Err(error)
        }
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
        if let Err(error) = responses::store(request_id, Value::Object(payload.clone())) {
            return error;
        }
        log_info(format!(
            "hotkey response received requestId={request_id} status={status} from={from}"
        ));
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
        Err(error) => return responses::reply(from, request_id, error),
    };
    let platform = match platform(payload) {
        Ok(value) => value,
        Err(error) => return responses::reply(from, request_id, error),
    };
    let Some(chord) = chord_for(shortcut_id, platform) else {
        return responses::reply(
            from,
            request_id,
            refused("badArgument", "shortcutId is not a built-in shortcut"),
        );
    };
    log_info(format!(
        "hotkey execute received requestId={request_id} shortcutId={shortcut_id} platform={platform} chord={chord} from={from}"
    ));
    let outcome = host::platform_simulate_keys(chord);
    match &outcome {
        Outcome::Ok(_) => log_info(format!(
            "hotkey platform simulate succeeded requestId={request_id} chord={chord}"
        )),
        Outcome::Err(error) => log_info(format!(
            "hotkey platform simulate failed requestId={request_id} chord={chord} code={} message={}",
            error.code, error.message
        )),
    }
    responses::reply(from, request_id, outcome)
}

#[cfg(test)]
mod tests {
    use super::chord_for;

    #[test]
    fn built_in_shortcuts_have_fixed_chords() {
        assert_eq!(chord_for("copy", "windows"), Some("^c"));
        assert_eq!(chord_for("paste", "linux"), Some("ctrl+v"));
        assert_eq!(chord_for("copy", "desktop"), Some("^c"));
        assert_eq!(chord_for("open-folder", "desktop"), Some("{LWIN}e"));
    }

    #[test]
    fn unknown_shortcuts_are_refused() {
        assert_eq!(chord_for("ctrl-alt-delete", "windows"), None);
    }
}
