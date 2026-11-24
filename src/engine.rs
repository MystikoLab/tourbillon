use std::{
    collections::HashMap,
    sync::{Arc, Mutex},
    thread::{self, JoinHandle},
    time::Duration,
};

use crate::{
    app::{Actions, App},
    error::ActionError,
    pasteboard::PasteboardItem,
};
use fend_core::Context;
use log::info;
use objc2::rc::Retained;
use objc2_app_kit::{NSWorkspace, NSWorkspaceOpenConfiguration};

pub struct Engine<'a> {
    queried_results: Vec<&'a dyn Actions>,
    query: String,
    actions: Vec<&'a dyn Actions>,
    workspace: Retained<NSWorkspace>,
    aliases: HashMap<String, Vec<String>>,
    ranking: HashMap<String, i32>,
    pub clipboard_history_items: Arc<Mutex<Vec<PasteboardItem>>>,
    background_tasks: Vec<JoinHandle<()>>,
}

impl<'a> Engine<'a> {
    pub fn new(actions: Vec<&'a dyn Actions>) -> Self {
        info!("Initialising engine once");

        Engine {
            actions,
            queried_results: Vec::new(),
            query: String::new(),
            workspace: NSWorkspace::sharedWorkspace(),
            aliases: HashMap::new(),
            ranking: HashMap::new(),
            clipboard_history_items: Arc::new(Mutex::new(Vec::new())),
            background_tasks: Vec::new(),
        }
    }

    pub fn start_clipboard_bg_task(&mut self) {
        let cb_hist = self.clipboard_history_items.clone();

        self.background_tasks.push(thread::spawn(move || {
            loop {
                thread::sleep(Duration::from_millis(100));
                let current_item = PasteboardItem::get_current_pasteboard_item();
                let Ok(mut cb_hist) = cb_hist.try_lock() else {
                    continue;
                };

                if cb_hist.last().cloned() != current_item
                    && let Some(has_item) = current_item
                {
                    cb_hist.push(has_item);
                }
            }
        }));
    }

    pub fn load_aliases_and_ranking(
        &mut self,
        aliases: HashMap<String, Vec<String>>,
        ranking: HashMap<String, i32>,
    ) {
        self.aliases = aliases;
        self.ranking = ranking;
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

    pub fn query_actions(&mut self, query: &str) -> Vec<&'a dyn Actions> {
        info!("Querying actions");
        self.query = query.to_string();
        let mut result_actions = self
            .actions
            .iter()
            .copied()
            .filter_map(|x| {
                let rank = x
                    .index_for_action()
                    .weigh_index(&self.aliases, &self.ranking)
                    .rank_against_query(query);

                if rank > 0 { Some((x, rank)) } else { None }
            })
            .collect::<Vec<(&dyn Actions, i32)>>();

        result_actions.sort_unstable_by_key(|x| (-x.1, x.0.index_for_action().name.len()));

        self.queried_results = result_actions.into_iter().map(|x| x.0).collect();
        self.queried_results.clone()
    }

    pub fn open_action(&mut self, index: usize) -> Result<(), ActionError> {
        let action = self
            .queried_results
            .get(index)
            .ok_or(ActionError::CouldNotFindAction)?;
        action.call_action();
        self.ranking
            .entry(action.index_for_action().name.to_lowercase())
            .and_modify(|x| *x += 1)
            .or_insert(1);

        Ok(())
    }

    pub fn get_math_result(&self, query: &str) -> Option<String> {
        info!("Querying for math result");
        fend_core::evaluate(query, &mut Context::new())
            .ok()
            .map(|x| x.get_main_result().to_string())
    }
}

#[cfg(test)]
mod tests {
    use crate::system::SystemAction;

    use super::*;
    #[test]
    fn system_actions_engine_test() {
        let acts = SystemAction::index();
        let mut eng = Engine::new(acts.iter().map(|x| x as _).collect());

        assert_eq!(eng.query_actions("quit").len(), 1);
        assert_eq!(eng.query_actions("slep").len(), 2);
        assert_eq!(eng.query_actions("sleep").len(), 2);
        assert_eq!(eng.query_actions("all").len(), 2);
        assert_eq!(eng.query_actions("s").len(), 6);
    }
}
