use vigem::XButton;

pub const XBUTTONS: [(&str, XButton); 11] = [
    ("A", XButton::A),
    ("B", XButton::B),
    ("X", XButton::X),
    ("Y", XButton::Y),
    ("LB", XButton::LeftShoulder),
    ("RB", XButton::RightShoulder),
    ("Back", XButton::Back),
    ("Start", XButton::Start),
    ("LS", XButton::LeftThumb),
    ("RS", XButton::RightThumb),
    ("Guide", XButton::Guide),
];

pub fn xbutton_names() -> Vec<&'static str> {
    XBUTTONS.iter().copied().map(|(name, _)| name).collect()
}

pub const GBUTTONS: [(&str, u16); 6] = [("A", 0x1), ("B", 0x2), ("X", 0x4), ("Y", 0x8), ("Z", 0x200), ("Start", 0x100)];

#[derive(Clone)]
pub struct Config {
    pub buttons: [usize; 6],
    pub auto_recenter: bool,
    pub deadzone: u8,
}

impl Default for Config {
    fn default() -> Self {
        Self { buttons: [0, 2, 1, 3, 5, 7], auto_recenter: true, deadzone: 5 }
    }
}
