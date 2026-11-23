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
    engine.query_actions("aari");

    let actions: Vec<&str> = engine
        .queried_results
        .clone()
        .iter()
        .map(|x| x.index_for_action().name)
        .collect();

    dbg!(actions);
}
