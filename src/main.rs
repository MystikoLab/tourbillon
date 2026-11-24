use std::thread;
use std::time::Duration;

use dotenvy::dotenv;

use crate::engine::Engine;
use crate::{
    app::{Actions, App},
    system::SystemAction,
};

pub mod app;
pub mod discovery;
pub mod engine;
pub mod index;
pub mod pasteboard;
pub mod system;

fn main() {
    dotenv().ok();
    env_logger::init();

    let apps = App::index_system_apps();
    let system_actions = SystemAction::index();

    let actions = apps
        .iter()
        .map(|x| x as &dyn Actions)
        .chain(system_actions.iter().map(|x| x as &dyn Actions))
        .collect();

    let mut engine = Engine::new(actions);

    engine.start_clipboard_bg_task();
    println!("Started clipboard bg task");

    thread::sleep(Duration::from_secs(2));

    dbg!(&engine.clipboard_history_items);
    thread::sleep(Duration::from_secs(2));

    dbg!(&engine.clipboard_history_items);
}
