const CONFIG_PATH: &str = "gc-adapter.ini";
mod section {
    pub const BUTTONS: &str = "Buttons";
    pub const CONTROLLER: &str = "Controller";
    pub const APPLICATION: &str = "Application";
}
mod item {
    pub const AUTO_RECENTER: &str = "AutoRecenter";
    pub const DEADZONE: &str = "Deadzone";
    pub const CLOSE_TO_TRAY: &str = "CloseToTray";
}

macro_rules! log {
    ($logger:expr, $fmt:literal) => {{
        $logger(concat!($fmt, "\r\n"))
    }};
    ($logger:expr, $fmt:literal, $($arg:expr),+) => {{
        $logger(&format!(concat!($fmt, "\r\n"), $($arg),+))
    }}
}

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
        Self { buttons: [0, 2, 1, 3, 5, 7], auto_recenter: false, deadzone: 5, close_to_tray: true }
    }
}

impl Config {
    pub fn load(logger: &impl Fn(&str)) -> Self {
        let mut config = Default::default();
        let ini = match ini::Ini::load_from_file(CONFIG_PATH) {
            Ok(ini) => ini,
            Err(ini::Error::Io(e)) if e.kind() == std::io::ErrorKind::NotFound => {
                log!(logger, "{} doesn't exist, using defaults.", CONFIG_PATH);
                return config
            },
            Err(e) => {
                log!(logger, "Couldn't load settings: {}\r\nUsing defaults.", e);
                return config
            },
        };
        if let Some(buttons) = ini.section(Some(section::BUTTONS)) {
            for (my_id, gbut_name) in config.buttons.iter_mut().zip(GBUTTONS.iter().copied().map(|(name, _)| name)) {
                if let Some(xbut_name) = buttons.get(gbut_name) {
                    if let Some(id) = XBUTTONS.iter().copied().position(|(name, _)| name == xbut_name) {
                        *my_id = id;
                    } else {
                        log!(logger, "Mapping for {} button is invalid ({}), using default", gbut_name, xbut_name);
                    }
                } else {
                    log!(logger, "Mapping for {} button not found, using default", gbut_name);
                }
            }
        } else {
            log!(logger, "Buttons section not found, using defaults");
        }
        fn load_bool(logger: &impl Fn(&str), section: &ini::Properties, out: &mut bool, name: &str) {
            if let Some(setting_str) = section.get(name) {
                if let Ok(setting_bool) = setting_str.parse() {
                    *out = setting_bool;
                } else {
                    log!(logger, "{} setting invalid ({}), using default", name, setting_str);
                }
            } else {
                log!(logger, "{} setting not found, using default", name);
            }
        }
        if let Some(section) = ini.section(Some(section::CONTROLLER)) {
            load_bool(logger, section, &mut config.auto_recenter, item::AUTO_RECENTER);
            if let Some(deadzone_str) = section.get(item::DEADZONE) {
                if let Some(deadzone_int) = deadzone_str.parse().ok().filter(|&i| i <= 100) {
                    config.deadzone = deadzone_int;
                } else {
                    log!(logger, "Deadzone setting invalid ({}), using default", deadzone_str);
                }
            } else {
                log!(logger, "Deadzone setting not found, using default");
            }
        } else {
            log!(logger, "Controller section not found, using defaults");
        }
        if let Some(section) = ini.section(Some(section::APPLICATION)) {
            load_bool(logger, section, &mut config.close_to_tray, item::CLOSE_TO_TRAY);
        } else {
            log!(logger, "Application section not found, using defaults");
        }
        log!(logger, "Settings loaded from {}.", CONFIG_PATH);
        config
    }

    pub fn save(&self, logger: &impl Fn(&str)) -> bool {
        let mut ini = ini::Ini::new();
        for (gc, xb) in self.buttons.iter().copied().enumerate() {
            ini.entry(Some(section::BUTTONS.into()))
                .or_insert_with(Default::default)
                .insert(GBUTTONS[gc].0, XBUTTONS[xb].0);
        }
        ini.with_section(Some(section::CONTROLLER))
            .set(item::AUTO_RECENTER, self.auto_recenter.to_string())
            .set(item::DEADZONE, self.deadzone.to_string());
        ini.with_section(Some(section::APPLICATION)).set(item::CLOSE_TO_TRAY, self.close_to_tray.to_string());
        match ini.write_to_file(CONFIG_PATH) {
            Ok(()) => {
                log!(logger, "Settings saved to {}.", CONFIG_PATH);
                true
            },
            Err(e) => {
                log!(logger, "Failed to save settings: {}", e);
                false
            },
        }
    }
}
