# Tourbillon is a rust based engine that launchers need.

Q: Why do we need a singular engine?
A: Faster to build launchers with comparable feature parity, and also a unified platform to add the security that launchers need, allowing multiple launchers to be safer, and also more featureful than everyone having their own implementation.

Q: Why Rust?
A: Rust is fast, supports the needed macOS APIs through the objc2 crates, and also offers the ability to use amazing GUI crates like GPUI if SwiftUI is undesired.

Q: What inspired you to make this?
A: There are so many different launchers, and all of them have their own disadvantages. For example, Rustcast just straight up looked like trash, and was missing so many features like extensions.

---

## Features

As of now, this engine supports:
- A few system actions
- searching installed apps (indexed via the LaunchServices Private API)
- Searching through the apps / actions.
- Perform maths and unit conversion (using the engine behind [fend](https://github.com/printfn/fend))
- Automatically add clipboard history items (files, images, urls, text and color)
- Automated ranking when using the `.open_action` method on the engine.
- Support for aliases and subaliases in the index.

This handles most of the "low level" macOS APIs without putting too much stress on the user.



## AI Usage:
- No AI used in writing this code,to achieve a neutral stance to let both Vibe coders and Anti AI coders use it.
- I expect all contributions to be the same.
- If it's your own fork, do as you wish. It's the point of a fork. But don't expect AI contributions to be merged, or even manually coded shit code.
