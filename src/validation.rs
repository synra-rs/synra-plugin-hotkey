use serde_json::{Map, Value};
use synra_plugin_sdk::{Error, Outcome};

use super::{MAX_PLATFORM, MAX_REQUEST_ID};

pub(super) fn target(value: &Map<String, Value>) -> Result<&str, Outcome> {
    let target = required_string(value, "target", 64)?;
    if !is_device_id(target) {
        return Err(refused(
            "badArgument",
            "target must be a canonical device id",
        ));
    }
    Ok(target)
}

pub(super) fn request_id(value: &Map<String, Value>) -> Result<&str, Outcome> {
    required_string(value, "requestId", MAX_REQUEST_ID)
}

pub(super) fn platform(value: &Map<String, Value>) -> Result<&str, Outcome> {
    let platform = required_string(value, "platform", MAX_PLATFORM)?;
    if !matches!(platform, "windows" | "linux" | "macos" | "desktop") {
        return Err(refused("badArgument", "platform is not supported"));
    }
    Ok(platform)
}

pub(super) fn required_string<'a>(
    value: &'a Map<String, Value>,
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

pub(super) fn is_device_id(value: &str) -> bool {
    value.len() == 36
        && value.as_bytes().iter().enumerate().all(|(index, byte)| {
            if matches!(index, 8 | 13 | 18 | 23) {
                *byte == b'-'
            } else {
                byte.is_ascii_hexdigit()
            }
        })
}

pub(super) fn refused(code: &str, message: &str) -> Outcome {
    Outcome::Err(Error {
        code: code.to_owned(),
        message: message.to_owned(),
        details: None,
    })
}
