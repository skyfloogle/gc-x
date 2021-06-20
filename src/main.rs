use vigem::{Target, TargetType, Vigem, XButton, XUSBReport};

mod adapter;

fn main() {
    let waiter = adapter::GCAdapterWaiter::new().unwrap();
    let adapter = waiter.try_connect_controller().unwrap().unwrap();
    let mut vigem = Vigem::new();
    vigem.connect().unwrap();
    let mut targets = [None, None, None, None];
    loop {
        let pads = adapter.get_pads();
        for (pad_opt, target_opt) in pads.iter().zip(&mut targets) {
            match (pad_opt, target_opt.as_ref()) {
                (Some(_), None) => {
                    let mut target = Target::new(TargetType::Xbox360);
                    vigem.target_add(&mut target).unwrap();
                    *target_opt = Some(target);
                }
                (None, Some(target)) => {
                    vigem.target_remove(target).unwrap();
                    *target_opt = None;
                }
                _ => (),
            }
            if let (Some(pad), Some(target)) = (pad_opt.as_ref(), target_opt.as_ref()) {
                let mut w_buttons = XButton::empty();
                let mut map_button = |bit: u16, but| {
                    if pad.buttons & (1 << bit) != 0 {
                        w_buttons.insert(but)
                    }
                };
                map_button(0, XButton::A);
                map_button(1, XButton::B);
                map_button(2, XButton::X);
                map_button(3, XButton::Y);
                map_button(4, XButton::DpadLeft);
                map_button(5, XButton::DpadRight);
                map_button(6, XButton::DpadDown);
                map_button(7, XButton::DpadUp);
                map_button(8, XButton::Start);
                map_button(9, XButton::RightShoulder);

                let deadstick = |ax| (i16::from(ax) - 0x80) << 8;

                let report = XUSBReport {
                    w_buttons,
                    b_left_trigger: pad.trigger_left,
                    b_right_trigger: pad.trigger_right,
                    s_thumb_lx: deadstick(pad.stick_x),
                    s_thumb_ly: deadstick(pad.stick_y),
                    s_thumb_rx: deadstick(pad.cstick_x),
                    s_thumb_ry: deadstick(pad.cstick_y),
                };
                vigem.update(&target, &report).unwrap();
            }
        }
    }
}
