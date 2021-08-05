#![windows_subsystem = "windows"]

use parking_lot::{Condvar, Mutex, Once};
use std::sync::Arc;

mod adapter;
mod config;
mod daemon;
mod ui;

#[macro_export]
macro_rules! log {
    ($logger:expr, $fmt:literal) => {{
        $logger.log(concat!($fmt, "\r\n"))
    }};
    ($logger:expr, $fmt:literal, $($arg:expr),+) => {{
        $logger.log(&format!(concat!($fmt, "\r\n"), $($arg),+))
    }}
}

fn main() {
    let exit_once = Arc::new(Once::new());

    let config = Arc::new(Mutex::new(Default::default()));

    let must_center = Arc::new(Mutex::new([false; 4]));

    let joy_connected = Arc::new(Mutex::new([false; 4]));

    let ui = match ui::init_app(exit_once.clone(), config.clone(), must_center.clone(), joy_connected.clone()) {
        Ok(ui) => ui,
        Err(e) => {
            ui::show_error("Could not initialize UI", &format!("Could not initialize UI: {}", e));
            return
        },
    };
    let logger = ui.logger.clone();

    let wait_for_init = Arc::new((Mutex::new(false), Condvar::new()));

    std::thread::Builder::new()
        .name("daemon".into())
        .spawn({
            let wait_for_init = wait_for_init.clone();
            let exit_once = exit_once.clone();
            let join_sender = ui.join_sender;
            let leave_sender = ui.leave_sender;
            let exit_sender = ui.exit_sender;
            move || {
                let mut daemon = match daemon::Daemon::new(
                    exit_once.clone(),
                    logger.clone(),
                    config,
                    must_center,
                    joy_connected,
                    join_sender,
                    leave_sender,
                    exit_sender,
                ) {
                    Ok(daemon) => daemon,
                    Err(()) => {
                        exit_once.call_once(|| ());
                        wait_for_init.1.notify_all();
                        return
                    },
                };
                *wait_for_init.0.lock() = true;
                wait_for_init.1.notify_all();
                daemon.run()
            }
        })
        .unwrap();

    // wait until daemon is created without errors before finishing ui
    {
        let mut lock = wait_for_init.0.lock();
        if !*lock {
            wait_for_init.1.wait(&mut lock);
        }
    }

    if exit_once.state().done() {
        return
    }

    ui::run_ui();
}
