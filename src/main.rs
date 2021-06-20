#![windows_subsystem = "windows"]

use parking_lot::{Mutex, Once};
use std::sync::Arc;
use vigem::{Target, TargetType, Vigem, XButton, XUSBReport};

mod adapter;
mod notification;
mod ui;

fn main() {
    let exit_once = Arc::new(Once::new());
    let deadzone: i16 = 0x100;
    let mapping = [
        XButton::A,             // A
        XButton::X,             // B
        XButton::B,             // X
        XButton::Y,             // Y
        XButton::DpadLeft,      // left
        XButton::DpadRight,     // right
        XButton::DpadDown,      // down
        XButton::DpadUp,        // up
        XButton::Start,         // start
        XButton::RightShoulder, // Z
    ];

    let waiter = match adapter::GCAdapterWaiter::new(exit_once.clone()) {
        Ok(waiter) => waiter,
        Err(rusb::Error::NotSupported) => {
            ui::show_error(
                "Error: GC Adapter driver not installed",
                "You haven't correctly installed the adapter driver.",
            );
            return
        },
        Err(e) => {
            ui::show_error("Error", &format!("Could not initialize libusb: {}", e));
            return
        },
    };
    let mut vigem = Vigem::new();
    if let Err(e) = vigem.connect() {
        match e {
            vigem::VigemError::BusNotFound => {
                ui::show_error("Error: VigEmBus not found", "VigEmBus not found. You may need to install it.")
            },
            e => ui::show_error("Error", &format!("Could not connect to VigEmBus: {}", e)),
        }
        return
    }
    let targets = Arc::new(Mutex::new([None, None, None, None]));
    let mut notif_handles = [None, None, None, None];
    let rumbles = Arc::new(Mutex::new([0; 4]));

    // start UI
    if let Err(e) = std::thread::Builder::new().name("GUI".into()).spawn({
        let exit_once = exit_once.clone();
        move || {
            let res = ui::init_app(exit_once.clone());
            exit_once.call_once(|| ());
            if let Err(e) = res {
                ui::show_error("Could not initialize UI", &format!("Could not initialize UI: {}", e));
            }
        }
    }) {
        ui::show_error("Could not start UI thread", &format!("Could not start UI thread: {}\nI will now exit.", e));
        return
    }

    if exit_once.state().done() {
        return
    }

    loop {
        let pads = waiter.get_pads();
        if exit_once.state().done() {
            break
        }
        for ((pad_opt, target_opt), notif) in pads.iter().zip(targets.lock().iter_mut()).zip(&mut notif_handles) {
            match (pad_opt, target_opt.as_ref()) {
                (Some(_), None) => {
                    println!("New GC controller connected!");
                    let mut target = Target::new(TargetType::Xbox360);
                    vigem.target_add(&mut target).unwrap();

                    let rumbles = rumbles.clone();
                    let targets = targets.clone();
                    let waiter = &waiter;
                    *notif = Some(
                        notification::register_notification(&mut vigem, &target, move |notif| {
                            let rumble = (u16::from(notif.large_motor) * 0x55
                                + u16::from(notif.small_motor) * (0x100 - 0x55))
                                > 0x800;
                            let i = targets.lock().iter().position(|tg: &Option<Target>| {
                                tg.as_ref().map(|tg| tg.index() == notif.get_target().index()).unwrap_or(false)
                            });
                            if let Some(i) = i {
                                let mut rumbles = rumbles.lock();
                                rumbles[i] = rumble.into();
                                waiter.send_rumble(rumbles.clone());
                            }
                        })
                        .unwrap(),
                    );
                    *target_opt = Some(target);
                },
                (None, Some(target)) => {
                    println!("GC controller disconnected.");
                    vigem.target_remove(target).unwrap();
                    *target_opt = None;
                    *notif = None;
                },
                _ => (),
            }
            if let (Some(pad), Some(target)) = (pad_opt.as_ref(), target_opt.as_mut()) {
                let mut w_buttons = XButton::empty();
                for (bit, but) in mapping.iter().enumerate() {
                    if pad.buttons & (1 << bit) != 0 {
                        w_buttons.insert(*but);
                    }
                }

                let deadstick = |ax| match (i16::from(ax) - 0x80 << 8) + i16::from(ax) {
                    ax if ax.abs() < deadzone => 0,
                    ax => ax,
                };

                let report = XUSBReport {
                    w_buttons,
                    b_left_trigger: pad.trigger_left,
                    b_right_trigger: pad.trigger_right,
                    s_thumb_lx: deadstick(pad.stick_x),
                    s_thumb_ly: deadstick(pad.stick_y),
                    s_thumb_rx: deadstick(pad.cstick_x),
                    s_thumb_ry: deadstick(pad.cstick_y),
                };
                target.update(&report).unwrap();
            }
        }
    }
}
