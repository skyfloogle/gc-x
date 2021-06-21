use crate::{adapter::GCAdapterWaiter, notification, ui};
use parking_lot::{Mutex, Once};
use std::sync::Arc;
use vigem::{Target, TargetType, Vigem, XButton, XUSBReport};

pub struct Daemon {
    exit_once: Arc<Once>,
    waiter: GCAdapterWaiter,
    vigem: Vigem,
    logger: ui::Logger,
}

impl Daemon {
    pub fn new(exit_once: Arc<Once>, logger: ui::Logger) -> Result<Self, ()> {
        // all fallible initialization goes here
        let waiter = match GCAdapterWaiter::new(exit_once.clone(), logger.clone()) {
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
        let mut vigem = Vigem::new();
        if let Err(e) = vigem.connect() {
            match e {
                vigem::VigemError::BusNotFound => {
                    ui::show_error("Error: VigEmBus not found", "VigEmBus not found. You may need to install it.")
                },
                e => ui::show_error("Error", &format!("Could not connect to VigEmBus: {}", e)),
            }
            return Err(())
        }
        Ok(Self { exit_once, waiter, vigem, logger })
    }

    pub fn run(&mut self) {
        let deadzone: i16 = 0xc00;
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

        let targets = Arc::new(Mutex::new([None, None, None, None]));
        let mut notif_handles = [None, None, None, None];
        let rumbles = Arc::new(Mutex::new([0; 4]));
        let mut centers: [((i16, i16), (i16, i16)); 4] = Default::default();

        let transform = |ax| (i16::from(ax) - 0x80 << 8) + i16::from(ax);

        loop {
            let pads = self.waiter.get_pads();
            if self.exit_once.state().done() {
                break
            }
            for (((pad_opt, target_opt), notif), center) in
                pads.iter().zip(targets.lock().iter_mut()).zip(&mut notif_handles).zip(&mut centers)
            {
                match (pad_opt, target_opt.as_ref()) {
                    (Some(pad), None) => {
                        self.logger.log("New GC controller connected!");
                        *center = (
                            (transform(pad.stick_x), transform(pad.stick_y)),
                            (transform(pad.cstick_x), transform(pad.cstick_y)),
                        );
                        let mut target = Target::new(TargetType::Xbox360);
                        self.vigem.target_add(&mut target).unwrap();

                        let rumbles = rumbles.clone();
                        let targets = targets.clone();
                        let waiter = &self.waiter;
                        *notif = Some(
                            notification::register_notification(&mut self.vigem, &target, move |notif| {
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
                        self.logger.log("GC controller disconnected.");
                        self.vigem.target_remove(target).unwrap();
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

                    let deadstick = |ax, center: i16| match transform(ax) - center {
                        ax if ax.abs() < deadzone => 0,
                        ax => ax,
                    };

                    let report = XUSBReport {
                        w_buttons,
                        b_left_trigger: pad.trigger_left,
                        b_right_trigger: pad.trigger_right,
                        s_thumb_lx: deadstick(pad.stick_x, center.0.0),
                        s_thumb_ly: deadstick(pad.stick_y, center.0.1),
                        s_thumb_rx: deadstick(pad.cstick_x, center.1.0),
                        s_thumb_ry: deadstick(pad.cstick_y, center.1.1),
                    };
                    target.update(&report).unwrap();
                }
            }
        }
    }
}
