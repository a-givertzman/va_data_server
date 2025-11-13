use std::env;
use once_cell::sync::Lazy;
use snap7_sys::LibSnap7;

static RED: &str = "\x1b[0;31m";
static YELLOW: &str ="\x1b[1;93m";
static NC: &str = "\x1b[0m"; // No Color

pub static S7LIB: Lazy<LibSnap7> = Lazy::new(|| {
    log::info!("LibSnap7.init | Initializing LibSnap7...");
    let paths = [
        format!("{}/libsnap7.so", env::current_dir().unwrap().display()),
        format!("{}/lib/libsnap7.so", env::current_dir().unwrap().display()),
        "/usr/lib/libsnap7.so".to_owned(),
    ];
    for path in paths {
        log::info!("LibSnap7.init | Check '{}'...", path);
        match unsafe { LibSnap7::new(&path) } {
            Ok(lib) => {
                log::info!("LibSnap7.init | check '{}' - ok", path);
                return lib;
            }
            Err(_) => {
                log::trace!("LibSnap7.init | initializing LibSnap7 | check '{}' - {}not found{}", path, YELLOW, NC);
            }
        }
    }
    log::info!("LibSnap7.init | {}initializing LibSnap7 - ERROR{}", RED, NC);
    panic!("libsnap7.so - not found")
});

