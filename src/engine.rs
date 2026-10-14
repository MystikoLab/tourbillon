use crate::app::{Actions, App};
use fend_core::Context;
use log::info;
use objc2::rc::Retained;
use objc2_app_kit::{NSWorkspace, NSWorkspaceOpenConfiguration};

pub struct Engine<'a> {
    actions: Vec<&'a dyn Actions>,
    pub workspace: Retained<NSWorkspace>,
}

impl<'a> Engine<'a> {
    pub fn new(actions: Vec<&'a dyn Actions>) -> Self {
        info!("Initialising engine once");

        Engine {
            actions,
            workspace: NSWorkspace::sharedWorkspace(),
        }
    }

    pub fn open_app(&self, app: &App) {
        self.workspace
            .openApplicationAtURL_configuration_completionHandler(
                &app.ns_url(),
                &NSWorkspaceOpenConfiguration::configuration(),
                None,
            );

        info!("Opening app")
    }

    pub fn query(&self, query: &str) -> Vec<&'a dyn Actions> {
        self.actions
            .iter()
            .copied()
            .filter(|x| {
                x.index_for_action()
                    .map(|x| x.name.to_lowercase().contains(query))
                    .unwrap_or(false)
            })
            .collect()
    }

    pub fn get_math_result(&self, query: &str) -> Option<String> {
        fend_core::evaluate(query, &mut Context::new())
            .ok()
            .map(|x| x.get_main_result().to_string())
    }
}
