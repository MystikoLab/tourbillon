//! macOS application discovery using Launch Services.
//!
//! This module uses the undocumented `LSCopyAllApplicationURLs` API to enumerate
//! all registered applications on the system. This private API has been stable
//! since macOS 10.5 and is widely used by launcher applications (Alfred, Raycast, etc.).
//!
//! Since the symbol is not exported in Apple's `.tbd` stub files (which only list
//! documented APIs), we load it at runtime via `dlsym` from the LaunchServices
//! framework. If loading fails, we fall back to the cross-platform directory
//! scanning approach.
//!
//! This has been copied over from <https://github.com/MystikoLab/rustcast/blob/master/src/platform/macos/discovery.rs>

use core::{
    ffi::{CStr, c_void},
    mem,
    ptr::{self, NonNull},
};
use std::{
    env, fs,
    path::{Path, PathBuf},
    sync::LazyLock,
};

use log::{error, info};
use objc2_core_foundation::{CFArray, CFRetained, CFURL};
use objc2_foundation::{NSBundle, NSDictionary, NSNumber, NSString, NSURL, ns_string};

use crate::app::App;

/// Function signature for `LSCopyAllApplicationURLs`.
///
/// This undocumented Launch Services function retrieves URLs for all applications
/// registered with the system. It follows Core Foundation's "Copy Rule" - the
/// caller owns the returned `CFArray` and is responsible for releasing it.
///
/// # Parameters
/// - `out`: Pointer to receive the `CFArray<CFURL>` of application URLs
///
/// # Returns
/// - `0` (`noErr`) on success
/// - Non-zero `OSStatus` error code on failure
type LSCopyAllApplicationURLsFn = unsafe extern "C" fn(out: *mut *const CFArray<CFURL>) -> i32;

/// Path to the LaunchServices framework binary within CoreServices.
const LAUNCHSERVICES_PATH: &CStr =
    c"/System/Library/Frameworks/CoreServices.framework/Frameworks/LaunchServices.framework/LaunchServices";

/// Logs the last `dlerror` message with a prefix.
///
/// # Safety
///
/// Must be called immediately after a failed `dlopen`/`dlsym` call,
/// before any other dl* functions are invoked.
unsafe fn log_dlerror(prefix: &str) {
    let error = unsafe { libc::dlerror() };
    let message = if error.is_null() {
        "unknown error".into()
    } else {
        unsafe { CStr::from_ptr(error) }.to_string_lossy()
    };

    error!("{prefix}: {message}");
}

/// Dynamically loads `LSCopyAllApplicationURLs` from the LaunchServices framework.
///
/// This function is called once and cached via `LazyLock`. We use dynamic loading
/// because the symbol is undocumented and not present in Apple's `.tbd` stub files,
/// which prevents static linking on modern macOS.
///
/// The library handle is intentionally kept open for the process lifetime since
/// we cache the function pointer.
///
/// # Returns
///
/// The function pointer if successfully loaded, `None` otherwise.
fn load_symbol() -> Option<LSCopyAllApplicationURLsFn> {
    // SAFETY: We pass a valid null-terminated path string to dlopen.
    // RTLD_NOW resolves symbols immediately; RTLD_LOCAL keeps them private.
    let lib = unsafe {
        libc::dlopen(
            LAUNCHSERVICES_PATH.as_ptr(),
            libc::RTLD_NOW | libc::RTLD_LOCAL,
        )
    };

    let Some(lib) = NonNull::new(lib) else {
        // SAFETY: dlopen has returned a null pointer, indicating failure.
        unsafe { log_dlerror("failed to load LaunchServices framework") };
        return None;
    };

    // Clear any prior error before checking dlsym result.
    unsafe { libc::dlerror() };

    // SAFETY: We pass a valid library handle and null-terminated symbol name.
    let sym = unsafe { libc::dlsym(lib.as_ptr(), c"_LSCopyAllApplicationURLs".as_ptr()) };
    let Some(sym) = NonNull::new(sym) else {
        // SAFETY: dlsym has returned a null pointer, indicating failure.
        unsafe { log_dlerror("failed to find symbol `LSCopyAllApplicationURLs`") };

        // SAFETY: lib is a valid handle from successful dlopen.
        unsafe { libc::dlclose(lib.as_ptr()) };
        return None;
    };

    // SAFETY: We've verified the symbol exists. The function signature matches
    // the known (though undocumented) API based on reverse engineering and
    // widespread usage in other applications.
    Some(unsafe { mem::transmute::<*mut c_void, LSCopyAllApplicationURLsFn>(sym.as_ptr()) })
}

/// Retrieves URLs for all applications registered with Launch Services.
///
/// Uses the cached function pointer from [`load_symbol`] to call the
/// undocumented `LSCopyAllApplicationURLs` API.
///
/// # Returns
///
/// `Some(CFRetained<CFArray<CFURL>>)` containing application URLs on success,
/// `None` if the symbol couldn't be loaded or the API call failed.
fn registered_app_urls() -> Option<CFRetained<CFArray<CFURL>>> {
    static SYM: LazyLock<Option<LSCopyAllApplicationURLsFn>> = LazyLock::new(load_symbol);

    let sym = (*SYM)?;
    let mut urls_ptr = ptr::null();

    // SAFETY: We've verified `sym` is a valid function pointer. We pass a valid
    // mutable pointer to receive the output. The function follows the "Copy Rule"
    // so we take ownership of the returned CFArray.
    let err = unsafe { sym(&mut urls_ptr) };

    if err != 0 {
        error!("LSCopyAllApplicationURLs failed with error code: {err}");
        return None;
    }

    let Some(url_ptr) = NonNull::new(urls_ptr.cast_mut()) else {
        error!("LSCopyAllApplicationURLs returned null on success");
        return None;
    };

    // SAFETY: LSCopyAllApplicationURLs returns a +1 retained CFArray on success.
    // We transfer ownership to CFRetained which will call CFRelease when dropped.
    Some(unsafe { CFRetained::from_raw(url_ptr) })
}

/// Directories that contain user-facing applications.
/// Apps in these directories are included by default (after LSUIElement check).
static USER_APP_DIRECTORIES: LazyLock<&'static [&'static Path]> = LazyLock::new(|| {
    // These strings live for the lifetime of the program, so are safe to leak.
    let items = [
        Path::new("/Applications/"),
        Path::new("/System/Applications/"),
    ];

    let Some(home) = env::var_os("HOME") else {
        return Box::leak(Box::new(items));
    };

    let home_apps = Path::new(&home).join("Applications/");
    let home_apps = PathBuf::leak(home_apps);

    Box::leak(Box::new([items[0], items[1], home_apps]))
});

#[derive(Debug, Clone, Default, PartialEq, Eq)]
struct AppBundleMetadata {
    bundle_display_name: Option<String>,
    bundle_name: Option<String>,
    application_category: Option<String>,
    background_only: bool,
}

fn is_in_user_app_directory_with<'a>(
    path: &Path,
    directories: impl IntoIterator<Item = &'a Path>,
) -> bool {
    directories
        .into_iter()
        .any(|directory| path.starts_with(directory))
}

fn should_include_app(path: &Path, metadata: &AppBundleMetadata) -> bool {
    should_include_app_with(path, metadata, USER_APP_DIRECTORIES.iter().copied())
}

fn should_include_app_with<'a>(
    path: &Path,
    metadata: &AppBundleMetadata,
    directories: impl IntoIterator<Item = &'a Path>,
) -> bool {
    if is_nested_inside_another_app(path) || is_helper_location(path) || metadata.background_only {
        return false;
    }

    if !is_in_user_app_directory_with(path, directories) && metadata.application_category.is_none()
    {
        return false;
    }

    true
}

fn select_app_name(path: &Path, metadata: &AppBundleMetadata) -> Option<String> {
    path.file_stem()
        .map(|stem| stem.to_string_lossy().into_owned())
        .or_else(|| metadata.bundle_display_name.clone())
        .or_else(|| metadata.bundle_name.clone())
}

fn extract_bundle_metadata(
    info: &NSDictionary<NSString, objc2::runtime::AnyObject>,
) -> AppBundleMetadata {
    let get_string = |key: &NSString| -> Option<String> {
        info.objectForKey(key)?
            .downcast::<NSString>()
            .ok()
            .map(|s| s.to_string())
    };

    let is_truthy = |key: &NSString| -> bool {
        info.objectForKey(key)
            .map(|v| {
                v.downcast_ref::<NSNumber>().is_some_and(|n| n.boolValue())
                    || v.downcast_ref::<NSString>().is_some_and(|s| {
                        s.to_string() == "1" || s.to_string().eq_ignore_ascii_case("YES")
                    })
            })
            .unwrap_or(false)
    };

    AppBundleMetadata {
        bundle_display_name: get_string(ns_string!("CFBundleDisplayName")),
        bundle_name: get_string(ns_string!("CFBundleName")),
        application_category: get_string(ns_string!("LSApplicationCategoryType")),
        background_only: is_truthy(ns_string!("LSBackgroundOnly")),
    }
}

/// Extracts application metadata from a bundle URL.
///
/// Queries the bundle's `Info.plist` for display name and icon, with the
/// following fallback chain for the app name:
/// 1. `CFBundleDisplayName` - localized display name
/// 2. `CFBundleName` - short bundle name
/// 3. File stem from path (e.g., "Safari" from "Safari.app")
///
/// # Returns
///
/// `Some(App)` if the bundle is valid and has a determinable name, `None` otherwise.
fn query_app(url: impl AsRef<NSURL>) -> Option<App> {
    let url = url.as_ref();
    let path = url.to_file_path()?;
    let bundle = NSBundle::bundleWithURL(url)?;
    let info = bundle.infoDictionary()?;
    let metadata = extract_bundle_metadata(&info);
    if !should_include_app(&path, &metadata) {
        return None;
    }

    let name = select_app_name(&path, &metadata)?;

    Some(App { name, path })
}

/// Returns all installed applications discovered via Launch Services.
///
/// Attempts to use the native `LSCopyAllApplicationURLs` API for comprehensive
/// app discovery. If the API is unavailable (symbol not found or call fails),
/// falls back to the cross-platform directory scanning approach.
///
/// # Arguments
///
/// * `store_icons` - Whether to load application icons (slower but needed for display)
pub fn get_installed_apps() -> Vec<App> {
    let Some(registered_app_urls) = registered_app_urls() else {
        error!("native app discovery unavailable, falling back to directory scan");
        return fallback_get_installed_apps();
    };

    // Intermediate allocation into a vec allows us to parallelize the iteration, speeding up discovery by ~5x.
    let urls: Vec<_> = registered_app_urls.into_iter().collect();

    urls.into_iter().filter_map(query_app).collect()
}

fn is_nested_inside_another_app(app_path: &Path) -> bool {
    // Walk up ancestors; if we find an *.app component that is NOT the last component,
    // then this app is nested inside another app bundle.
    let comps: Vec<_> = app_path.components().collect();
    // Normalize: if path ends with ".../Foo.app", we look for any earlier "*.app".
    for component in comps.iter().take(comps.len().saturating_sub(1)) {
        if let std::path::Component::Normal(name) = component
            && name.to_string_lossy().ends_with(".app")
        {
            return true;
        }
    }
    false
}

fn is_helper_location(path: &Path) -> bool {
    let s = path.to_string_lossy();
    s.contains("/Contents/Library/LoginItems/")
        || s.contains("/Contents/XPCServices/")
        || s.contains("/Contents/Helpers/")
        || s.contains("/Contents/Frameworks/")
        || s.contains("/Library/PrivilegedHelperTools/")
}

pub fn default_app_paths() -> [String; 4] {
    let user_local_path = std::env::var("HOME").unwrap() + "/Applications/";

    [
        "/Applications/".to_string(),
        user_local_path,
        "/System/Applications/".to_string(),
        "/System/Applications/Utilities/".to_string(),
    ]
}

pub(crate) fn fallback_get_installed_apps() -> Vec<App> {
    default_app_paths()
        .into_iter()
        .flat_map(discover_apps)
        .collect()
}

/// This gets all the installed apps in the given directory
///
/// Is a fallback from the method in [`crate::platform::macos::discovery::get_installed_apps`]
///
/// the directories are defined in [`crate::app::tile::elm::new`]
fn discover_apps(dir: impl AsRef<Path>) -> Vec<App> {
    info!("Indexing apps started");
    let entries = match fs::read_dir(dir.as_ref()) {
        Ok(entries) => entries.filter_map(Result::ok).collect::<Vec<_>>(),
        Err(error) => {
            error!(
                "Could not read directry: {} because of:\n{}",
                dir.as_ref().to_string_lossy(),
                error
            );
            return Vec::new();
        }
    };

    entries
        .into_iter()
        .filter_map(move |x| {
            let file_type = match x.file_type() {
                Ok(file_type) => file_type,
                Err(error) => {
                    error!("Unable to map entries: {}", error);
                    return None;
                }
            };
            if !file_type.is_dir() {
                return None;
            }

            let file_name_os = x.file_name();
            let file_name = file_name_os.to_string_lossy().to_string();

            if !file_name.ends_with(".app") {
                return None;
            }

            let path = x.path();

            let name = file_name.strip_suffix(".app").unwrap().to_string();
            Some(App {
                name,
                path,
            })
        })
        .collect()
}
