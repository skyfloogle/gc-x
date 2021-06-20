use native_windows_derive::NwgUi;
use native_windows_gui as nwg;
use nwg::NativeUi;
use parking_lot::Once;
use std::sync::Arc;

#[derive(Default, NwgUi)]
pub struct App {
    #[nwg_resource()]
    embed_resource: nwg::EmbedResource,

    #[nwg_resource(source_embed: Some(&data.embed_resource), source_embed_str: Some("icon"))]
    icon: nwg::Icon,

    #[nwg_control(title: "gc-adapter", icon: Some(&data.icon), flags: "WINDOW")]
    #[nwg_events(OnInit: [App::show_welcome], OnWindowClose: [App::exit])]
    window: nwg::Window,

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
}

impl App {
    fn right_click(&self) {
        let (x, y) = nwg::GlobalCursor::position();
        self.tray_popup.popup(x, y);
    }

    fn show_welcome(&self) {
        self.tray.show("gc-adapter runs via the taskbar.", None, None, None);
    }

    fn exit(&self) {
        nwg::stop_thread_dispatch();
    }
}

pub fn init_app(exit_once: Arc<Once>) -> Result<(), nwg::NwgError> {
    nwg::init()?;
    let _app = App::build_ui(Default::default())?;
    nwg::dispatch_thread_events();
    exit_once.call_once(|| ());
    Ok(())
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
