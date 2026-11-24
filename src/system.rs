use std::ptr;

use objc2::{AnyThread, MainThreadMarker, msg_send, runtime::AnyObject};
use objc2_app_kit::NSApplication;
use objc2_foundation::{NSAppleScript, NSString};

use crate::{app::Actions, index::RawIndex};

pub enum SystemAction {
    QuitSelf,
    Sleep,
    Shutdown,
    Restart,
    Logout,
    EmptyTrash,
    EjectAllDisks,
    SleepAllDisplays,
}

impl SystemAction {
    pub fn perform(&self) {
        match self {
            SystemAction::QuitSelf => quit_self(),
            SystemAction::Shutdown => shutdown(),
            SystemAction::Sleep => sleep(),
            SystemAction::Restart => restart(),
            SystemAction::Logout => logout(),
            SystemAction::EmptyTrash => empty_trash(),
            SystemAction::EjectAllDisks => eject_all_disks(),
            SystemAction::SleepAllDisplays => sleep_all_displays(),
        }
    }

    pub fn name(&self) -> &str {
        match self {
            SystemAction::QuitSelf => "quit",
            SystemAction::Sleep => "sleep",
            SystemAction::Shutdown => "shutdown",
            SystemAction::Restart => "restart",
            SystemAction::Logout => "logout",
            SystemAction::EmptyTrash => "emptytrash",
            SystemAction::EjectAllDisks => "ejectalldisks",
            SystemAction::SleepAllDisplays => "sleepalldisplays",
        }
    }
    pub fn index() -> Vec<SystemAction> {
        vec![
            SystemAction::Sleep,
            SystemAction::QuitSelf,
            SystemAction::Shutdown,
            SystemAction::Restart,
            SystemAction::Logout,
            SystemAction::EmptyTrash,
            SystemAction::EjectAllDisks,
            SystemAction::SleepAllDisplays,
        ]
    }
}

fn sleep_all_displays() {
    std::process::Command::new("pmset").arg("displaysleepnow");
}

fn quit_self() {
    let app = NSApplication::sharedApplication(
        MainThreadMarker::new().expect("Quitting self must be called on the main thread"),
    );

    app.terminate(None);
}

fn sleep() {
    system_events_osascript("sleep");
}

fn shutdown() {
    system_events_osascript("shut down");
}

fn restart() {
    system_events_osascript("restart");
}

fn logout() {
    system_events_osascript("log out");
}

fn empty_trash() {
    finder_events_osascript("empty trash");
}

fn eject_all_disks() {
    finder_events_osascript("eject (every disk whose ejectable is true)");
}

fn system_events_osascript(action: &str) {
    let src = NSString::from_str(&format!("tell application \"System Events\" to {}", action));
    let script = NSAppleScript::initWithSource(NSAppleScript::alloc(), &src).unwrap();
    unsafe {
        let _: *mut AnyObject =
            msg_send![&script, executeAndReturnError: ptr::null_mut::<*mut AnyObject>()];
    }
}

fn finder_events_osascript(action: &str) {
    let src = NSString::from_str(&format!("tell application \"Finder\" to {}", action));
    let script = NSAppleScript::initWithSource(NSAppleScript::alloc(), &src).unwrap();
    unsafe {
        let _: *mut AnyObject =
            msg_send![&script, executeAndReturnError: ptr::null_mut::<*mut AnyObject>()];
    }
}

impl Actions for SystemAction {
    fn call_action(&self) {
        self.perform();
    }

    fn index_for_action(&self) -> RawIndex<'_> {
        RawIndex { name: self.name() }
    }
}
