use std::collections::HashMap;

use crate::app::{Actions, App};
use fend_core::Context;
use log::info;
use objc2::rc::Retained;
use objc2_app_kit::{NSWorkspace, NSWorkspaceOpenConfiguration};

pub struct Engine<'a> {
    pub queried_results: Vec<&'a dyn Actions>,
    query: String,
    pub actions: Vec<&'a dyn Actions>,
    workspace: Retained<NSWorkspace>,
    aliases: HashMap<String, Vec<String>>,
    ranking: HashMap<String, i32>,
}

impl<'a> Engine<'a> {
    pub fn new(actions: Vec<&'a dyn Actions>) -> Self {
        info!("Initialising engine once");

        Engine {
            actions,
            queried_results: Vec::new(),
            query: "".to_string(),
            workspace: NSWorkspace::sharedWorkspace(),
            aliases: HashMap::new(),
            ranking: HashMap::new(),
        }
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

    pub fn query_actions(&mut self, query: &str) {
        info!("Querying actions");
        self.query = query.to_string();
        let query = query;
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
    }

    pub fn open_action(&mut self, index: usize) -> Result<(), ()> {
        let action = self.queried_results.get(index).ok_or(())?;
        action.call_action();
        self.ranking
            .entry(action.index_for_action().name.to_string())
            .and_modify(|x| *x += 1)
            .or_insert(0);

        Ok(())
    }

    pub fn get_math_result(&self, query: &str) -> Option<String> {
        fend_core::evaluate(query, &mut Context::new())
            .ok()
            .map(|x| x.get_main_result().to_string())
    }
}
