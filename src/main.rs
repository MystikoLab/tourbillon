use dotenvy::dotenv;

use tourbillon::engine::Engine;
use tourbillon::{
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

    let apps = App::index();
    let system_actions = SystemAction::index();

    let actions = apps
        .iter()
        .map(|x| x as &dyn Actions)
        .chain(system_actions.iter().map(|x| x as &dyn Actions))
        .collect();

    let engine = Engine::new(actions);
    let result = engine.query("a");

    for i in result {
        dbg!(i.index_for_action());
    }

    engine.get_math_result("log(10 * 10)");
}
