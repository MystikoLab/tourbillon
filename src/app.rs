use fend_core::FendResult;
use log::info;
use objc2::rc::Retained;
use objc2_app_kit::NSWorkspace;
use objc2_foundation::NSURL;
use std::path::PathBuf;

use crate::index::Index;

#[derive(Debug, Clone)]
pub struct App {
    pub name: String,
    pub path: PathBuf,
}

impl App {
    pub fn index() -> Vec<Self> {
        info!("Indexing apps");
        crate::discovery::get_installed_apps()
    }

    pub(crate) fn ns_url(&self) -> Retained<NSURL> {
        info!("getting NSURL for app");
        NSURL::from_path(&self.path, true, None).expect("Unable to convert to a NSURL")
    }
}

pub trait Actions {
    fn call_action(&self);

    fn index_for_action(&self) -> Option<Index<'_>>;
}

impl Actions for App {
    fn call_action(&self) {
        NSWorkspace::sharedWorkspace().openURL(&self.ns_url());
    }

    fn index_for_action(&self) -> Option<Index<'_>> {
        Some(Index { name: &self.name })
    }
}

impl Actions for FendResult {
    fn call_action(&self) {
        println!("{}", self.get_main_result());
        todo!("Copy to clipboard functionality for actions for fend result")
    }

    fn index_for_action(&self) -> Option<Index<'_>> {
        None
    }
}
