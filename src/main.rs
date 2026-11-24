use dotenvy::dotenv;

use crate::engine::Engine;
use crate::{
    app::{Actions, App},
    system::SystemAction,
};

pub mod app;
pub mod discovery;
pub mod engine;
pub mod error;
pub mod index;
pub mod pasteboard;
pub mod system;

fn main() {
    dotenv().ok();
    env_logger::init();

    let apps = App::index_system_apps();
    dbg!(apps.len());
    let system_actions = SystemAction::index();
    dbg!(system_actions.len());

    let actions = apps
        .iter()
        .map(|x| x as &dyn Actions)
        .chain(system_actions.iter().map(|x| x as &dyn Actions))
        .collect();

    let mut engine = Engine::new(actions);

    engine.query_actions("s").len();
}
