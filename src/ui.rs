use crate::config::{xbutton_names, Config};
use native_windows_derive::{NwgPartial, NwgUi};
use native_windows_gui as nwg;
use native_windows_gui::{
    stretch::{
        geometry::Size,
        style::{Dimension, FlexDirection},
    },
    CheckBoxState,
};
use nwg::NativeUi;
use parking_lot::{Mutex, Once};
use std::sync::Arc;
use winapi::um::playsoundapi::{PlaySoundA, SND_ALIAS_ID, SND_ASYNC};

const FULL_SIZE: Size<Dimension> = Size { width: Dimension::Percent(1.0), height: Dimension::Percent(1.0) };

#[derive(Default, NwgPartial)]
pub struct Port {
    #[nwg_layout()]
    layout: nwg::GridLayout,

    #[nwg_control(text: "Deadzone")]
    #[nwg_layout_item(layout: layout, col: 0, row: 0)]
    deadzone_label: nwg::Label,

    #[nwg_control]
    #[nwg_layout_item(layout: layout, col: 1, row: 0)]
    deadzone_text: nwg::TextInput,

    #[nwg_control()]
    #[nwg_layout_item(layout: layout, col: 0, row: 1, col_span: 2)]
    deadzone_slider: nwg::TrackBar,

    #[nwg_control(text: "Button mapping")]
    #[nwg_layout_item(layout: layout, col: 0, row: 2, col_span: 2)]
    map_label: nwg::Label,

    #[nwg_control(text: "A")]
    #[nwg_layout_item(layout: layout, col: 0, row: 3)]
    a_label: nwg::Label,

    #[nwg_control(collection: xbutton_names(), selected_index: Some(0))]
    #[nwg_layout_item(layout: layout, col: 0, row: 4)]
    a_map: nwg::ComboBox<&'static str>,

    #[nwg_control(text: "B")]
    #[nwg_layout_item(layout: layout, col: 1, row: 3)]
    b_label: nwg::Label,

    #[nwg_control(collection: xbutton_names(), selected_index: Some(2))]
    #[nwg_layout_item(layout: layout, col: 1, row: 4)]
    b_map: nwg::ComboBox<&'static str>,

    #[nwg_control(text: "X")]
    #[nwg_layout_item(layout: layout, col: 0, row: 5)]
    x_label: nwg::Label,

    #[nwg_control(collection: xbutton_names(), selected_index: Some(1))]
    #[nwg_layout_item(layout: layout, col: 0, row: 6)]
    x_map: nwg::ComboBox<&'static str>,

    #[nwg_control(text: "Y")]
    #[nwg_layout_item(layout: layout, col: 1, row: 5)]
    y_label: nwg::Label,

    #[nwg_control(collection: xbutton_names(), selected_index: Some(3))]
    #[nwg_layout_item(layout: layout, col: 1, row: 6)]
    y_map: nwg::ComboBox<&'static str>,

    #[nwg_control(text: "Z")]
    #[nwg_layout_item(layout: layout, col: 0, row: 7)]
    z_label: nwg::Label,

    #[nwg_control(collection: xbutton_names(), selected_index: Some(5))]
    #[nwg_layout_item(layout: layout, col: 0, row: 8)]
    z_map: nwg::ComboBox<&'static str>,

    #[nwg_control(text: "Start")]
    #[nwg_layout_item(layout: layout, col: 1, row: 7)]
    st_label: nwg::Label,

    #[nwg_control(collection: xbutton_names(), selected_index: Some(7))]
    #[nwg_layout_item(layout: layout, col: 1, row: 8)]
    st_map: nwg::ComboBox<&'static str>,

    #[nwg_control(text: "Recenter:")]
    #[nwg_layout_item(layout: layout, col: 0, row: 9)]
    recenter_label: nwg::Label,

    #[nwg_control(text: "On join")]
    #[nwg_layout_item(layout: layout, col: 1, row: 9)]
    recenter_check: nwg::CheckBox,

    #[nwg_control(text: "P1", enabled: false)]
    #[nwg_layout_item(layout: layout, col: 0, row: 10)]
    recenter_p1: nwg::Button,

    #[nwg_control(text: "P2", enabled: false)]
    #[nwg_layout_item(layout: layout, col: 1, row: 10)]
    recenter_p2: nwg::Button,

    #[nwg_control(text: "P3", enabled: false)]
    #[nwg_layout_item(layout: layout, col: 0, row: 11)]
    recenter_p3: nwg::Button,

    #[nwg_control(text: "P4", enabled: false)]
    #[nwg_layout_item(layout: layout, col: 1, row: 11)]
    recenter_p4: nwg::Button,

    #[nwg_control(text: "Collapse to tray")]
    #[nwg_layout_item(layout: layout, col: 0, row: 12, col_span: 2)]
    tray_check: nwg::CheckBox,

    #[nwg_control(text: "Revert changes", enabled: false)]
    #[nwg_layout_item(layout: layout, col: 0, row: 13, col_span: 2)]
    revert_button: nwg::Button,

    #[nwg_control(text: "Save changes", enabled: false)]
    #[nwg_layout_item(layout: layout, col: 0, row: 14, col_span: 2)]
    save_button: nwg::Button,
}

#[derive(Default, NwgUi)]
pub struct App {
    #[nwg_resource]
    embed_resource: nwg::EmbedResource,

    #[nwg_resource(source_embed: Some(&data.embed_resource), source_embed_str: Some("icon"))]
    icon: nwg::Icon,

    #[nwg_control(title: "GC-X", icon: Some(&data.icon))]
    #[nwg_events(OnInit: [App::show_welcome], OnWindowClose: [App::close_window])]
    window: nwg::Window,

    #[nwg_layout(parent: window, flex_direction: FlexDirection::Row)]
    main_layout: nwg::FlexboxLayout,

    #[nwg_control(parent: window, flags: "VISIBLE")]
    #[nwg_layout_item(layout: main_layout, size: FULL_SIZE, max_size: FULL_SIZE)]
    log_frame: nwg::Frame,

    #[nwg_layout(parent: log_frame, flex_direction: FlexDirection::Column)]
    log_layout: nwg::FlexboxLayout,

    // give the label an absolute height, then the textbox takes 100% of what's left i guess
    #[nwg_control(text: "Log")]
    #[nwg_layout_item(layout: log_layout, size: Size { width: Dimension::Auto, height: Dimension::Points(10.0) })]
    log_label: nwg::Label,

    #[nwg_control(readonly: true)]
    #[nwg_layout_item(layout: log_layout, size: FULL_SIZE, max_size: FULL_SIZE)]
    log: nwg::TextBox,

    #[nwg_control]
    #[nwg_events(OnNotice: [App::update_log])]
    log_notice: nwg::Notice,

    log_buf: Arc<Mutex<String>>,

    #[nwg_control()]
    #[nwg_layout_item(
        layout: main_layout,
        size: Size { width: Dimension::Points(250.0), height: Dimension::Auto },
        min_size: Size { width: Dimension::Points(165.0), height: Dimension::Points(400.0) }
    )]
    port_frame: nwg::Frame,

    #[nwg_partial(parent: port_frame)]
    #[nwg_events(
        (deadzone_text, OnTextInput): [App::change_deadzone_textbox],
        (deadzone_slider, OnHorizontalScroll): [App::change_deadzone_slider],
        (recenter_p1, OnButtonClick): [App::recenter_p1],
        (recenter_p2, OnButtonClick): [App::recenter_p2],
        (recenter_p3, OnButtonClick): [App::recenter_p3],
        (recenter_p4, OnButtonClick): [App::recenter_p4],
        (a_map, OnComboxBoxSelection): [App::modify],
        (b_map, OnComboxBoxSelection): [App::modify],
        (x_map, OnComboxBoxSelection): [App::modify],
        (y_map, OnComboxBoxSelection): [App::modify],
        (z_map, OnComboxBoxSelection): [App::modify],
        (tray_check, OnButtonClick): [App::modify],
        (recenter_check, OnButtonClick): [App::modify],
        (revert_button, OnButtonClick): [App::revert_config],
        (save_button, OnButtonClick): [App::save_config],
    )]
    port: Port,

    #[nwg_control(popup: true)]
    tray_popup: nwg::Menu,

    #[nwg_control(parent: tray_popup, text: "GC-X")]
    #[nwg_events(OnMenuItemSelected: [App::revive_window])]
    popup_title: nwg::MenuItem,

    #[nwg_control(parent: tray_popup)]
    sep: nwg::MenuSeparator,

    #[nwg_control(parent: tray_popup, text: "GitHub")]
    #[nwg_events(OnMenuItemSelected: [App::website])]
    popup_website: nwg::MenuItem,

    #[nwg_control(parent: tray_popup, text: "Exit")]
    #[nwg_events(OnMenuItemSelected: [App::exit])]
    exit_item: nwg::MenuItem,

    #[nwg_control(tip: Some("GC-X"), icon: Some(&data.icon))]
    #[nwg_events(OnContextMenu: [App::right_click], MousePressLeftUp: [App::revive_window])]
    pub tray: nwg::TrayNotification,

    #[nwg_control]
    #[nwg_events(OnNotice: [App::controller_join])]
    pub join_notice: nwg::Notice,

    #[nwg_control]
    #[nwg_events(OnNotice: [App::controller_leave])]
    pub leave_notice: nwg::Notice,

    #[nwg_control]
    #[nwg_events(OnNotice: [App::exit])]
    pub exit_notice: nwg::Notice,

    pub exit_once: Arc<Once>,

    config: Arc<Mutex<Config>>,
    deadzone: Mutex<u8>,

    must_center: Arc<Mutex<[bool; 4]>>,
    joy_connected: Arc<Mutex<[bool; 4]>>,
}

impl App {
    fn right_click(&self) {
        let (x, y) = nwg::GlobalCursor::position();
        self.tray_popup.popup(x, y);
    }

    fn revive_window(&self) {
        self.window.restore();
    }

    fn close_window(&self) {
        if self.config.lock().close_to_tray {
            self.tray.show(
                "You can bring back the UI by clicking the icon.",
                Some("GC-X collapsed to tray"),
                None,
                None,
            );
        } else {
            self.exit();
        }
    }

    fn show_welcome(&self) {
        self.revert_config();
        self.port.revert_button.set_enabled(false);
        self.port.save_button.set_enabled(false);
    }

    fn modify(&self) {
        self.port.revert_button.set_enabled(true);
        self.port.save_button.set_enabled(true);
        // if it's already locked then it's being modified elsewhere
        if let Some(mut config) = self.config.try_lock() {
            for (but, cb) in config.buttons.iter_mut().zip([
                &self.port.a_map,
                &self.port.b_map,
                &self.port.x_map,
                &self.port.y_map,
                &self.port.z_map,
                &self.port.st_map,
            ]) {
                if let Some(sel) = cb.selection() {
                    *but = sel;
                }
            }
            config.deadzone = *self.deadzone.lock();
            config.auto_recenter = self.port.recenter_check.check_state() == CheckBoxState::Checked;
            config.close_to_tray = self.port.tray_check.check_state() == CheckBoxState::Checked;
        }
    }

    fn set_deadzone(&self, new_deadzone: u8, set_textbox: bool) {
        if let Some(mut deadzone) = self.deadzone.try_lock() {
            *deadzone = new_deadzone;
            self.port.deadzone_slider.set_pos(new_deadzone as _);
            if set_textbox {
                self.port.deadzone_text.set_text(&new_deadzone.to_string());
            }
            drop(deadzone); // avoid deadlocks
            self.modify();
        }
    }

    fn change_deadzone_textbox(&self) {
        let text = self.port.deadzone_text.text();
        if !text.is_empty() {
            match text.parse() {
                Ok(deadzone) if deadzone <= 100 => self.set_deadzone(deadzone, false),
                _ => unsafe {
                    // play windows asterisk sound
                    PlaySoundA(0x4453 as _, std::ptr::null_mut(), SND_ASYNC | SND_ALIAS_ID);
                },
            }
        }
    }

    fn change_deadzone_slider(&self) {
        self.set_deadzone(self.port.deadzone_slider.pos() as _, true);
    }

    fn revert_config(&self) {
        let new_config = Config::load(&|text| self.log(text));
        let mut config = self.config.lock();
        *config = new_config;
        self.set_deadzone(config.deadzone, true);
        for (but, cb) in config.buttons.iter().zip([
            &self.port.a_map,
            &self.port.b_map,
            &self.port.x_map,
            &self.port.y_map,
            &self.port.z_map,
            &self.port.st_map,
        ]) {
            cb.set_selection(Some(*but));
        }
        self.port.recenter_check.set_check_state(if config.auto_recenter {
            CheckBoxState::Checked
        } else {
            CheckBoxState::Unchecked
        });
        self.port.tray_check.set_check_state(if config.close_to_tray {
            CheckBoxState::Checked
        } else {
            CheckBoxState::Unchecked
        });
        self.port.revert_button.set_enabled(false);
        self.port.save_button.set_enabled(false);
    }

    fn save_config(&self) {
        if self.config.lock().save(&|text| self.log(text)) {
            self.port.revert_button.set_enabled(false);
            self.port.save_button.set_enabled(false);
        }
    }

    /// Send a log message. Message should end in CRLF.
    fn log(&self, text: &str) {
        print!("{}", text);
        fn replace_msg(textbox: &nwg::TextBox, text: &str) {
            use std::os::windows::ffi::OsStrExt;
            use winapi::um::winuser::EM_REPLACESEL;
            // convert to utf-16
            let osstr: &std::ffi::OsStr = text.as_ref();
            let utf: Vec<_> = osstr.encode_wide().chain(Some(0u16).into_iter()).collect();
            // paste new text at the end
            unsafe {
                winapi::um::winuser::SendMessageW(
                    textbox.handle.hwnd().unwrap(),
                    EM_REPLACESEL as u32,
                    false.into(),
                    utf.as_ptr() as _,
                );
            }
        }

        let sel = self.log.selection();

        let old_text = self.log.text();
        // 30k is apparently the maximum
        const MAX_LOG_LEN: usize = 30000;
        // if this would go past the maximum length, snip the start
        // will probably malfunction if message len is >30k but i won't be doing that
        let snipped_len = if old_text.len() + text.len() >= MAX_LOG_LEN {
            let unsnipped_len = old_text
                .lines()
                .rev()
                .scan(text.len(), |acc, line| {
                    *acc += line.len() + 2; // each line doesn't include CRLF
                    (*acc < MAX_LOG_LEN).then(|| *acc)
                })
                .last()
                .unwrap_or(0);
            let snipped_len = ((old_text.len() + text.len()) - unsnipped_len) as u32;
            self.log.set_selection(0..snipped_len);
            replace_msg(&self.log, "");
            snipped_len
        } else {
            0
        };
        let end = old_text.len() as u32 - snipped_len;

        // move cursor to end
        self.log.set_selection(end..end);
        replace_msg(&self.log, text);
        // move selection back to where it was before
        self.log.set_selection(sel.start.saturating_sub(snipped_len)..sel.end.saturating_sub(snipped_len));
    }

    fn update_log(&self) {
        let mut text = self.log_buf.lock();
        self.log(&mut text);
        text.clear();
    }

    fn recenter(&self, id: usize) {
        self.must_center.lock()[id] = true;
    }

    fn recenter_p1(&self) {
        self.recenter(0);
    }

    fn recenter_p2(&self) {
        self.recenter(1);
    }

    fn recenter_p3(&self) {
        self.recenter(2);
    }

    fn recenter_p4(&self) {
        self.recenter(3);
    }

    fn controller_join(&self) {
        let joy_connected = self.joy_connected.lock();
        self.port.recenter_p1.set_enabled(joy_connected[0]);
        self.port.recenter_p2.set_enabled(joy_connected[1]);
        self.port.recenter_p3.set_enabled(joy_connected[2]);
        self.port.recenter_p4.set_enabled(joy_connected[3]);
    }

    fn controller_leave(&self) {
        let joy_connected = self.joy_connected.lock();
        self.port.recenter_p1.set_enabled(joy_connected[0]);
        self.port.recenter_p2.set_enabled(joy_connected[1]);
        self.port.recenter_p3.set_enabled(joy_connected[2]);
        self.port.recenter_p4.set_enabled(joy_connected[3]);
    }

    fn website(&self) {
        const WEBSITE: &str = "https://github.com/skyfloogle/gc-x";
        if let Err(e) = open::that(WEBSITE) {
            self.log(&format!("Couldn't open website: {}\r\nCopy this link instead:\r\n{}\r\n", e, WEBSITE));
        }
    }

    fn exit(&self) {
        nwg::stop_thread_dispatch();
        self.exit_once.call_once(|| ());
    }
}

pub struct UiInfo {
    pub app: app_ui::AppUi,
    pub logger: Logger,
    pub join_sender: nwg::NoticeSender,
    pub leave_sender: nwg::NoticeSender,
    pub exit_sender: nwg::NoticeSender,
}

pub fn run_ui() {
    nwg::dispatch_thread_events();
}

pub fn init_app(
    exit_once: Arc<Once>,
    config: Arc<Mutex<Config>>,
    must_center: Arc<Mutex<[bool; 4]>>,
    joy_connected: Arc<Mutex<[bool; 4]>>,
) -> Result<UiInfo, nwg::NwgError> {
    nwg::init()?;
    let app = App {
        embed_resource: Default::default(),
        icon: Default::default(),
        window: Default::default(),
        main_layout: Default::default(),
        log_frame: Default::default(),
        log_layout: Default::default(),
        log_label: Default::default(),
        log: Default::default(),
        log_notice: Default::default(),
        log_buf: Arc::new(Default::default()),
        port_frame: Default::default(),
        port: Default::default(),
        tray_popup: Default::default(),
        popup_title: Default::default(),
        sep: Default::default(),
        popup_website: Default::default(),
        exit_item: Default::default(),
        tray: Default::default(),
        join_notice: Default::default(),
        leave_notice: Default::default(),
        exit_notice: Default::default(),
        exit_once,
        config,
        deadzone: Default::default(),
        must_center,
        joy_connected,
    };
    let app = App::build_ui(app)?;
    let logger = Logger { buf: app.log_buf.clone(), sender: app.log_notice.sender() };
    let join_sender = app.join_notice.sender();
    let leave_sender = app.leave_notice.sender();
    let exit_sender = app.exit_notice.sender();
    Ok(UiInfo { app, logger, join_sender, leave_sender, exit_sender })
}

pub fn show_error(title: &str, content: &str) {
    println!("Showing error: {}", content);
    nwg::message(&nwg::MessageParams {
        title,
        content,
        buttons: nwg::MessageButtons::Ok,
        icons: nwg::MessageIcons::Error,
    });
}

#[derive(Clone)]
pub struct Logger {
    buf: Arc<Mutex<String>>,
    sender: nwg::NoticeSender,
}

impl Logger {
    pub fn log(&self, text: &str) {
        let mut buf = self.buf.lock();
        buf.push_str(text);
        self.sender.notice();
    }
}
