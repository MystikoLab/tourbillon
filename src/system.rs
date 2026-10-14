use objc2::{AnyThread, MainThreadMarker};
use objc2_app_kit::NSApplication;
use objc2_foundation::{NSAppleScript, NSString};

use crate::{app::Actions, index::Index};

pub enum SystemAction {
    QuitSelf,
    Sleep,
    Shutdown,
}

impl SystemAction {
    pub fn perform(&self) {
        match self {
            SystemAction::QuitSelf => quit_self(),
            SystemAction::Shutdown => shutdown(),
            SystemAction::Sleep => sleep(),
        }
    }

    pub fn name(&self) -> &str {
        match self {
            SystemAction::QuitSelf => "quit",
            SystemAction::Sleep => "sleep",
            SystemAction::Shutdown => "shutdown",
        }
    }
    pub fn index() -> Vec<SystemAction> {
        vec![
            SystemAction::Sleep,
            SystemAction::QuitSelf,
            SystemAction::Shutdown,
        ]
    }
}

fn quit_self() {
    let app = NSApplication::sharedApplication(
        MainThreadMarker::new().expect("Quitting self must be called on the main thread"),
    );

    app.terminate(None);
}

fn sleep() {
    let src = NSString::from_str(r#"tell application "System Events" to sleep"#);
    let script = NSAppleScript::initWithSource(NSAppleScript::alloc(), &src).unwrap();
    let mut err = None;
    unsafe {
        script.executeAndReturnError(Some(&mut err));
    }
}

fn shutdown() {
    let src = NSString::from_str(r#"tell application "System Events" to shut down"#);
    let script = NSAppleScript::initWithSource(NSAppleScript::alloc(), &src).unwrap();
    let mut err = None;
    unsafe {
        script.executeAndReturnError(Some(&mut err));
    }
}

impl Actions for SystemAction {
    fn call_action(&self) {
        match self {
            Self::QuitSelf => {}

            Self::Sleep => sleep(),

            Self::Shutdown => shutdown(),
        }
    }

    fn index_for_action(&self) -> Option<Index<'_>> {
        Some(Index { name: self.name() })
    }
}
