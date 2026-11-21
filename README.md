# Tourbillon is a rust based engine that launchers need.

Q: Why do we need a singular engine?
A: Faster to build launchers with comparable feature parity, and also a unified platform to add the security that launchers need, allowing multiple launchers to be safer, and also more featureful than everyone having their own implementation.

Q: Why Rust?
A: Rust is fast, supports the needed macOS APIs through the objc2 crates, and also offers the ability to use amazing GUI crates like GPUI if SwiftUI is undesired.

Q: What inspired you to make this?
A: There are so many different launchers, and all of them have their own disadvantages. For example, Rustcast just straight up looked like trash, and was missing so many features like extensions.

---

## How to use this?

As of now, this engine supports:
- 3 system actions
- App searching (indexed via the LaunchServices Private API)
- Searching through the apps / actions.
- Perform maths and unit conversion (using the engine behind [fend](https://github.com/printfn/fend))

Take a look at main.rs:
```rs
fn main() {
    let apps = App::index(); // access the apps index. useful for getting the apps paths to load their images
    let system_actions = SystemAction::index(); // system actions like sleeping, shutdown, and quitting the app itself

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

    let math_result = engine.get_math_result("log(10 * 10)"); 
}
```

> Quick rundown:
you define what the user should be able to search through by creating an engine, which holds a reference to anything that implements `Actions`.
This allows you to build your own custom actions and add the relevant code for it, and query it with the same ranking and aliases support (yet to be added) easily. 

This handles most of the "low level" macOS APIs without putting too much stress on the user.

## AI Usage:
- No AI used in writing this code,to achieve a neutral stance to let both Vibe coders and Anti AI coders use it.
- I expect all contributions to be the same.
- If it's your own fork, do as you wish. It's the point of a fork. But don't expect AI contributions to be merged, or even manually coded shit code.
