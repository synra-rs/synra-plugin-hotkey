//! Stateful hotkey actions and the desktop receiver.

use synra_plugin_sdk::{Error, Outcome, exports};

struct Plugin;

impl exports::synra::plugin::lifecycle::Guest for Plugin {
    fn activate(_config_json: String) -> Outcome {
        crate::actions::declare()
    }

    fn deactivate() {}

    fn migrate(_from_version: String, _to_version: String) -> Outcome {
        Outcome::Ok(String::new())
    }

    fn on_tick(_tick_json: String) -> Outcome {
        Outcome::Ok(String::new())
    }

    fn on_event(event_json: String) -> Outcome {
        crate::actions::on_event(&event_json)
    }
}

impl exports::synra::plugin::share::Guest for Plugin {
    fn can_handle(_item_json: String) -> String {
        r#"{"handled":false}"#.to_owned()
    }

    fn execute(_action: String, _args_json: String) -> Outcome {
        refused()
    }
}

impl exports::synra::plugin::actions::Guest for Plugin {
    fn invoke(name: String, args_json: String) -> Outcome {
        crate::actions::invoke(&name, &args_json)
    }
}

fn refused() -> Outcome {
    Outcome::Err(Error {
        code: "actionNotFound".to_owned(),
        message: "this plugin has not implemented a share action".to_owned(),
        details: None,
    })
}

synra_plugin_sdk::export!(Plugin);
