use native_windows_derive::{NwgPartial, NwgUi};
use native_windows_gui as nwg;
use native_windows_gui::stretch::{
    geometry::Size,
    style::{Dimension, FlexDirection},
};
use nwg::NativeUi;
use parking_lot::{Mutex, Once};
use std::sync::Arc;

const FULL_SIZE: Size<Dimension> = Size { width: Dimension::Percent(1.0), height: Dimension::Percent(1.0) };

const BUTTONS: [&str; 10] = ["A", "B", "X", "Y", "LB", "RB", "Back", "Start", "LS", "RS"];

#[derive(Default, NwgPartial)]
pub struct Port {
    #[nwg_layout(flex_direction: FlexDirection::Column)]
    layout: nwg::FlexboxLayout,

    #[nwg_control(text: "Deadzone")]
    #[nwg_layout_item(layout: layout)]
    deadzone_label: nwg::Label,

    #[nwg_control(flags: "VISIBLE")]
    #[nwg_layout_item(layout: layout)]
    deadzone_frame: nwg::Frame,

    #[nwg_control(parent: deadzone_frame, value_int: 5, min_int: 0, max_int: 100)]
    deadzone_select: nwg::NumberSelect,

    #[nwg_control(text: "A")]
    #[nwg_layout_item(layout: layout)]
    a_label: nwg::Label,

    #[nwg_control(collection: BUTTONS.to_vec(), selected_index: Some(0))]
    #[nwg_layout_item(layout: layout)]
    a_map: nwg::ComboBox<&'static str>,

    #[nwg_control(text: "B")]
    #[nwg_layout_item(layout: layout)]
    b_label: nwg::Label,

    #[nwg_control(collection: BUTTONS.to_vec(), selected_index: Some(2))]
    #[nwg_layout_item(layout: layout)]
    b_map: nwg::ComboBox<&'static str>,

    #[nwg_control(text: "X")]
    #[nwg_layout_item(layout: layout)]
    x_label: nwg::Label,

    #[nwg_control(collection: BUTTONS.to_vec(), selected_index: Some(1))]
    #[nwg_layout_item(layout: layout)]
    x_map: nwg::ComboBox<&'static str>,

    #[nwg_control(text: "Y")]
    #[nwg_layout_item(layout: layout)]
    y_label: nwg::Label,

    #[nwg_control(collection: BUTTONS.to_vec(), selected_index: Some(3))]
    #[nwg_layout_item(layout: layout)]
    y_map: nwg::ComboBox<&'static str>,

    #[nwg_control(text: "Z")]
    #[nwg_layout_item(layout: layout)]
    z_label: nwg::Label,

    #[nwg_control(collection: BUTTONS.to_vec(), selected_index: Some(5))]
    #[nwg_layout_item(layout: layout)]
    z_map: nwg::ComboBox<&'static str>,

    #[nwg_control(text: "Start")]
    #[nwg_layout_item(layout: layout)]
    st_label: nwg::Label,

    #[nwg_control(collection: BUTTONS.to_vec(), selected_index: Some(7))]
    #[nwg_layout_item(layout: layout)]
    st_map: nwg::ComboBox<&'static str>,
}

#[derive(Default, NwgUi)]
pub struct App {
    #[nwg_resource]
    embed_resource: nwg::EmbedResource,

    #[nwg_resource(source_embed: Some(&data.embed_resource), source_embed_str: Some("icon"))]
    icon: nwg::Icon,

    #[nwg_control(title: "gc-adapter", icon: Some(&data.icon))]
    #[nwg_events(OnInit: [App::show_welcome], OnWindowClose: [App::exit])]
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
    #[nwg_layout_item(layout: main_layout, size: Size { width: Dimension::Points(175.0), height: Dimension::Auto })]
    port_frame: nwg::Frame,

    #[nwg_partial(parent: port_frame)]
    port: Port,

    #[nwg_control(popup: true)]
    tray_popup: nwg::Menu,

    #[nwg_control(parent: tray_popup, text: "gc-adapter", disabled: true)]
    popup_title: nwg::MenuItem,

    #[nwg_control(parent: tray_popup)]
    sep: nwg::MenuSeparator,

    #[nwg_control(parent: tray_popup, text: "Exit")]
    #[nwg_events(OnMenuItemSelected: [App::exit])]
    exit_item: nwg::MenuItem,

    #[nwg_control(tip: Some("gc-adapter"), icon: Some(&data.icon))]
    #[nwg_events(OnContextMenu: [App::right_click(SELF)])]
    pub tray: nwg::TrayNotification,

    #[nwg_control]
    #[nwg_events(OnNotice: [App::controller_join])]
    pub join_notice: nwg::Notice,

    #[nwg_control]
    #[nwg_events(OnNotice: [App::controller_leave])]
    pub leave_notice: nwg::Notice,

    pub exit_once: Arc<Once>,
}

impl App {
    fn right_click(&self) {
        let (x, y) = nwg::GlobalCursor::position();
        self.tray_popup.popup(x, y);
    }

    fn show_welcome(&self) {
        self.tray.show("gc-adapter runs via the taskbar.", None, None, None);
    }

    fn update_log(&self) {
        use std::os::windows::ffi::OsStrExt;
        use winapi::um::winuser::EM_REPLACESEL;

        let sel = self.log.selection();

        // move cursor to end (log.len() won't include the newlines so it's bad)
        let end = self.log.text().len() as u32;
        self.log.set_selection(end..end);
        // get text
        let mut text = self.log_buf.lock();
        // convert to utf-16
        let osstr: &std::ffi::OsStr = text.as_ref();
        let utf: Vec<_> = osstr.encode_wide().chain(Some(0u16).into_iter()).collect();
        // paste new text at the end
        unsafe {
            winapi::um::winuser::SendMessageW(
                self.log.handle.hwnd().unwrap(),
                EM_REPLACESEL as u32,
                false.into(),
                utf.as_ptr() as _,
            );
        }
        // clear buffer
        text.clear();
        // move selection back to where it was before
        self.log.set_selection(sel);
    }

    fn controller_join(&self) {
        self.tray.show("New controller connected", None, None, None);
    }

    fn controller_leave(&self) {
        self.tray.show("Controller disconnected", None, None, None);
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
}

pub fn run_ui() {
    nwg::dispatch_thread_events();
}

pub fn init_app(exit_once: Arc<Once>) -> Result<UiInfo, nwg::NwgError> {
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
        exit_item: Default::default(),
        tray: Default::default(),
        join_notice: Default::default(),
        leave_notice: Default::default(),
        exit_once,
    };
    let app = App::build_ui(app)?;
    let logger = Logger { buf: app.log_buf.clone(), sender: app.log_notice.sender() };
    let join_sender = app.join_notice.sender();
    let leave_sender = app.leave_notice.sender();
    Ok(UiInfo { app, logger, join_sender, leave_sender })
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
        println!("{}", text);
        let mut buf = self.buf.lock();
        buf.push_str(text);
        buf.push_str("\r\n");
        self.sender.notice();
    }
}
