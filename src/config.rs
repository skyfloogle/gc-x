bitflags::bitflags! {
    pub struct XButton: u16 {
        const DPAD_UP = 0x0001;
        const DPAD_DOWN = 0x0002;
        const DPAD_LEFT = 0x0004;
        const DPAD_RIGHT = 0x0008;
        const START = 0x0010;
        const BACK = 0x0020;
        const LEFT_THUMB = 0x0040;
        const RIGHT_THUMB = 0x0080;
        const LEFT_SHOULDER = 0x0100;
        const RIGHT_SHOULDER = 0x0200;
        const GUIDE = 0x0400;
        const A = 0x1000;
        const B = 0x2000;
        const X = 0x4000;
        const Y = 0x8000;
    }
}

bitflags::bitflags! {
    pub struct GButton: u16 {
        const A = 0x0001;
        const B = 0x0002;
        const X = 0x0004;
        const Y = 0x0008;
        const DPAD_LEFT = 0x0010;
        const DPAD_RIGHT = 0x0020;
        const DPAD_DOWN = 0x0040;
        const DPAD_UP = 0x0080;
        const START = 0x0100;
        const Z = 0x0200;
        const R = 0x0400;
        const L = 0x0800;
    }
}

pub const XBUTTONS: [(&str, XButton); 11] = [
    ("A", XButton::A),
    ("B", XButton::B),
    ("X", XButton::X),
    ("Y", XButton::Y),
    ("LB", XButton::LEFT_SHOULDER),
    ("RB", XButton::RIGHT_SHOULDER),
    ("Back", XButton::BACK),
    ("Start", XButton::START),
    ("LS", XButton::LEFT_THUMB),
    ("RS", XButton::RIGHT_THUMB),
    ("Guide", XButton::GUIDE),
];

pub fn xbutton_names() -> Vec<&'static str> {
    XBUTTONS.iter().copied().map(|(name, _)| name).collect()
}

pub const GBUTTONS: [(&str, GButton); 6] = [
    ("A", GButton::A),
    ("B", GButton::B),
    ("X", GButton::X),
    ("Y", GButton::Y),
    ("Z", GButton::Z),
    ("Start", GButton::START),
];

#[derive(Clone)]
pub struct Config {
    pub buttons: [usize; 6],
    pub auto_recenter: bool,
    pub deadzone: u8,
    pub close_to_tray: bool,
}

impl Default for Config {
    fn default() -> Self {
        Self { buttons: [0, 2, 1, 3, 5, 7], auto_recenter: true, deadzone: 5, close_to_tray: true }
    }
}
