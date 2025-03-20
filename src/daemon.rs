use crate::{
    adapter::GCAdapterWaiter,
    config::{self, Config, GButton, XButton},
    log, ui,
};
use native_windows_gui as nwg;
use parking_lot::{Mutex, Once};
use std::sync::Arc;
use vigem::{Target, UsbReport};

const INFO_STRINGS: [&str; 2] =
    [concat!("GC-X v", env!("CARGO_PKG_VERSION")), concat!("Built on ", env!("BUILD_DATE"), " #", env!("GIT_HASH"))];

pub struct Daemon {
    exit_once: Arc<Once>,
    waiter: GCAdapterWaiter,
    vigem: vigem::Client,
    logger: ui::Logger,
    config: Arc<Mutex<Config>>,
    must_center: Arc<Mutex<[bool; 4]>>,
    joy_connected: Arc<Mutex<[bool; 4]>>,
    join_sender: nwg::NoticeSender,
    leave_sender: nwg::NoticeSender,
}

impl Daemon {
    pub fn new(
        exit_once: Arc<Once>,
        logger: ui::Logger,
        config: Arc<Mutex<Config>>,
        must_center: Arc<Mutex<[bool; 4]>>,
        joy_connected: Arc<Mutex<[bool; 4]>>,
        join_sender: nwg::NoticeSender,
        leave_sender: nwg::NoticeSender,
        exit_sender: nwg::NoticeSender,
    ) -> Result<Self, ()> {
        // all fallible initialization goes here
        INFO_STRINGS.iter().for_each(|s| log!(logger, "{}", s));
        let waiter = match GCAdapterWaiter::new(exit_once.clone(), logger.clone(), exit_sender) {
            Ok(waiter) => waiter,
            Err(rusb::Error::NotSupported) => {
                ui::show_error(
                    "Error: GC Adapter driver not installed",
                    "You haven't correctly installed the adapter driver.",
                );
                return Err(())
            },
            Err(e) => {
                ui::show_error("Error", &format!("Could not initialize libusb: {}", e));
                return Err(())
            },
        };
        let mut vigem = vigem::Client::new();
        if let Err(e) = vigem.connect() {
            match e {
                vigem::Error::BusNotFound => {
                    ui::show_error("Error: VigEmBus not found", "VigEmBus not found. You may need to install it.")
                },
                e => ui::show_error("Error", &format!("Could not connect to VigEmBus: {}", e)),
            }
            return Err(())
        }
        Ok(Self { exit_once, waiter, vigem, logger, config, must_center, joy_connected, join_sender, leave_sender })
    }

    pub fn run(&mut self) {
        let targets = Arc::new(Mutex::new([None, None, None, None]));
        let mut notif_handles = [None, None, None, None];
        let rumbles = Arc::new(Mutex::new([0; 4]));
        let mut centers: [((i16, i16), (i16, i16)); 4] = Default::default();

        let transform = |ax| (i16::from(ax) - 0x80 << 8) + i16::from(ax);

        'outer: loop {
            let pads = self.waiter.get_pads();
            if self.exit_once.state().done() {
                break
            }
            for (i, pad_opt, target_opt, notif, center, must_center, connected) in itertools::izip!(
                0..,
                &pads,
                targets.lock().iter_mut(),
                &mut notif_handles,
                &mut centers,
                self.must_center.lock().iter_mut(),
                self.joy_connected.lock().iter_mut()
            ) {
                match (pad_opt, target_opt.as_mut()) {
                    (Some(_), None) => {
                        log!(self.logger, "New GC controller connected in port {}!", i + 1);
                        *center = ((0, 0), (0, 0));
                        *must_center = self.config.lock().auto_recenter;
                        *connected = true;
                        self.join_sender.notice();
                        let mut target = Target::new();
                        if let Err(e) = self.vigem.add_target(&mut target) {
                            log!(self.logger, "Could not add target: {}", e);
                            continue 'outer
                        }

                        let rumbles = rumbles.clone();
                        let targets = targets.clone();
                        let waiter = &self.waiter;
                        *notif = match target.register_notification(
                            move |target: &Target, large_motor: u8, small_motor: u8| {
                                let rumble =
                                    (u16::from(large_motor) * 0x55 + u16::from(small_motor) * (0x100 - 0x55)) > 0x800;
                                let i = targets.lock().iter().position(|tg: &Option<Target>| {
                                    tg.as_ref().map(|tg| tg.index() == target.index()).unwrap_or(false)
                                });
                                if let Some(i) = i {
                                    let mut rumbles = rumbles.lock();
                                    rumbles[i] = rumble.into();
                                    waiter.send_rumble(rumbles.clone());
                                }
                            },
                        ) {
                            Ok(handle) => Some(handle),
                            Err(e) => {
                                log!(self.logger, "Could not register rumble notification: {}", e);
                                None
                            },
                        };
                        *target_opt = Some(target);
                    },
                    (None, Some(target)) => {
                        log!(self.logger, "GC controller in port {} disconnected.", i + 1);
                        if let Err(e) = self.vigem.remove_target(target) {
                            log!(self.logger, "Failed to remove target: {}", e);
                        }
                        *target_opt = None;
                        *notif = None;
                        *connected = false;
                        self.leave_sender.notice();
                    },
                    _ => (),
                }
                if let (Some(pad), Some(target)) = (pad_opt.as_ref(), target_opt.as_mut()) {
                    let mut buttons = XButton::empty();
                    // dpad
                    for (gc, xb) in [
                        (GButton::DPAD_LEFT, XButton::DPAD_LEFT),
                        (GButton::DPAD_RIGHT, XButton::DPAD_RIGHT),
                        (GButton::DPAD_UP, XButton::DPAD_UP),
                        (GButton::DPAD_DOWN, XButton::DPAD_DOWN),
                    ] {
                        if pad.buttons.contains(gc) {
                            buttons.insert(xb);
                        }
                    }
                    for (gc, xb) in self.config.lock().buttons.iter().enumerate() {
                        if pad.buttons.contains(config::GBUTTONS[gc].1) {
                            buttons.insert(config::XBUTTONS[*xb].1);
                        }
                    }

                    if *must_center {
                        *center = (
                            (transform(pad.stick_x), transform(pad.stick_y)),
                            (transform(pad.cstick_x), transform(pad.cstick_y)),
                        );
                        *must_center = false;
                        log!(self.logger, "Joysticks centered for P{}", i + 1);
                    }

                    let deadstick = |ax, center: i16| match transform(ax) - center {
                        ax if ax.abs()
                            < (f32::from(i16::MAX) * f32::from(self.config.lock().deadzone) / 100.0) as _ =>
                        {
                            0
                        },
                        ax => ax,
                    };

                    let report = UsbReport {
                        buttons: buttons.bits(),
                        left_trigger: pad.trigger_left,
                        right_trigger: pad.trigger_right,
                        left_x: deadstick(pad.stick_x, center.0 .0),
                        left_y: deadstick(pad.stick_y, center.0 .1),
                        right_x: deadstick(pad.cstick_x, center.1 .0),
                        right_y: deadstick(pad.cstick_y, center.1 .1),
                    };
                    if let Err(e) = target.update(&report) {
                        log!(self.logger, "Failed to update target: {}", e);
                    }
                }
            }
        }
    }
}
