#![allow(dead_code)]

use std::{ops::DerefMut, pin::Pin};

mod bindings {
    #![allow(non_upper_case_globals)]
    #![allow(non_camel_case_types)]
    #![allow(non_snake_case)]
    #![allow(deref_nullptr)]

    #[link(name = "setupapi")]
    extern "C" {}

    include!("bindings.rs");
}

pub enum Error {
    None,
    BusNotFound,
    NoFreeSlot,
    InvalidTarget,
    RemovalFailed,
    AlreadyConnected,
    TargetUninitialized,
    TargetNotPluggedIn,
    BusVersionMismatch,
    BusAccessFailed,
    CallbackAlreadyRegistered,
    CallbackNotFound,
    BusAlreadyConnected,
    BusInvalidHandle,
    XusbUserIndexOutOfRange,
    InvalidParameter,
    NotSupported,
}

impl Error {
    fn new(err: bindings::VIGEM_ERROR) -> Self {
        match err {
            bindings::_VIGEM_ERRORS_VIGEM_ERROR_NONE => Self::None,
            bindings::_VIGEM_ERRORS_VIGEM_ERROR_BUS_NOT_FOUND => Self::BusNotFound,
            bindings::_VIGEM_ERRORS_VIGEM_ERROR_NO_FREE_SLOT => Self::NoFreeSlot,
            bindings::_VIGEM_ERRORS_VIGEM_ERROR_INVALID_TARGET => Self::InvalidTarget,
            bindings::_VIGEM_ERRORS_VIGEM_ERROR_REMOVAL_FAILED => Self::RemovalFailed,
            bindings::_VIGEM_ERRORS_VIGEM_ERROR_ALREADY_CONNECTED => Self::AlreadyConnected,
            bindings::_VIGEM_ERRORS_VIGEM_ERROR_TARGET_UNINITIALIZED => Self::TargetUninitialized,
            bindings::_VIGEM_ERRORS_VIGEM_ERROR_TARGET_NOT_PLUGGED_IN => Self::TargetNotPluggedIn,
            bindings::_VIGEM_ERRORS_VIGEM_ERROR_BUS_VERSION_MISMATCH => Self::BusVersionMismatch,
            bindings::_VIGEM_ERRORS_VIGEM_ERROR_BUS_ACCESS_FAILED => Self::BusAccessFailed,
            bindings::_VIGEM_ERRORS_VIGEM_ERROR_CALLBACK_ALREADY_REGISTERED => Self::CallbackAlreadyRegistered,
            bindings::_VIGEM_ERRORS_VIGEM_ERROR_CALLBACK_NOT_FOUND => Self::CallbackNotFound,
            bindings::_VIGEM_ERRORS_VIGEM_ERROR_BUS_ALREADY_CONNECTED => Self::BusAlreadyConnected,
            bindings::_VIGEM_ERRORS_VIGEM_ERROR_BUS_INVALID_HANDLE => Self::BusInvalidHandle,
            bindings::_VIGEM_ERRORS_VIGEM_ERROR_XUSB_USERINDEX_OUT_OF_RANGE => Self::XusbUserIndexOutOfRange,
            bindings::_VIGEM_ERRORS_VIGEM_ERROR_INVALID_PARAMETER => Self::InvalidParameter,
            bindings::_VIGEM_ERRORS_VIGEM_ERROR_NOT_SUPPORTED => Self::NotSupported,
            _ => unreachable!(),
        }
    }

    fn into_result(self) -> Result<()> {
        match self {
            Self::None => Ok(()),
            err => Err(err),
        }
    }
}

impl std::fmt::Display for Error {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::None => write!(f, "no error"),
            Self::BusNotFound => write!(f, "bus not found"),
            Self::NoFreeSlot => write!(f, "no free slot"),
            Self::InvalidTarget => write!(f, "invalid target"),
            Self::RemovalFailed => write!(f, "removal failed"),
            Self::AlreadyConnected => write!(f, "already connected"),
            Self::TargetUninitialized => write!(f, "target uninitialized"),
            Self::TargetNotPluggedIn => write!(f, "target not plugged in"),
            Self::BusVersionMismatch => write!(f, "bus version mismatch"),
            Self::BusAccessFailed => write!(f, "bus access failed"),
            Self::CallbackAlreadyRegistered => write!(f, "callback already registered"),
            Self::CallbackNotFound => write!(f, "callback not found"),
            Self::BusAlreadyConnected => write!(f, "bus already connected"),
            Self::BusInvalidHandle => write!(f, "bus invalid handle"),
            Self::XusbUserIndexOutOfRange => write!(f, "xusb user index out of range"),
            Self::InvalidParameter => write!(f, "invalid parameter"),
            Self::NotSupported => write!(f, "not supported"),
        }
    }
}

type Result<T> = std::result::Result<T, Error>;

pub struct Client(bindings::PVIGEM_CLIENT);

impl Client {
    pub fn new() -> Self {
        unsafe { Self(bindings::vigem_alloc()) }
    }

    pub fn connect(&mut self) -> Result<()> {
        unsafe { Error::new(bindings::vigem_connect(self.0)).into_result() }
    }

    pub fn disconnect(&mut self) {
        unsafe { bindings::vigem_disconnect(self.0) }
    }

    pub fn add_target(&self, target: &mut Target) -> Result<()> {
        unsafe {
            Error::new(bindings::vigem_target_add(self.0, target.target)).into_result()?;
        }
        target.client = Some(self.0);
        Ok(())
    }

    pub fn remove_target(&self, target: &mut Target) -> Result<()> {
        unsafe {
            Error::new(bindings::vigem_target_remove(self.0, target.target)).into_result()?;
        }
        target.client = None;
        Ok(())
    }
}

impl Drop for Client {
    fn drop(&mut self) {
        unsafe {
            self.disconnect();
            bindings::vigem_free(self.0);
        }
    }
}

pub struct NotificationHandle<'a> {
    target: bindings::PVIGEM_TARGET,
    func: Box<dyn FnMut(&Target, u8, u8) + 'a>,
}

pub struct Target {
    target: bindings::PVIGEM_TARGET,
    is_ref: bool,
    client: Option<bindings::PVIGEM_CLIENT>,
}

#[repr(C)]
pub struct UsbReport {
    pub buttons: u16,
    pub left_trigger: u8,
    pub right_trigger: u8,
    pub left_x: i16,
    pub left_y: i16,
    pub right_x: i16,
    pub right_y: i16,
}

impl UsbReport {
    fn to_raw(&self) -> bindings::XUSB_REPORT {
        unsafe { *std::mem::transmute::<&Self, &bindings::XUSB_REPORT>(self) }
    }
}

unsafe extern "stdcall" fn handle_notification(
    _client: bindings::PVIGEM_CLIENT,
    target: bindings::PVIGEM_TARGET,
    large_motor: bindings::UCHAR,
    small_motor: bindings::UCHAR,
    _led_number: bindings::UCHAR,
    user_data: bindings::LPVOID,
) {
    let handle = &mut *user_data.cast::<NotificationHandle>();
    (handle.func)(&Target::new_ref(target), large_motor, small_motor);
}

impl Target {
    pub fn new() -> Self {
        unsafe { Self { target: bindings::vigem_target_x360_alloc(), is_ref: false, client: None } }
    }

    fn new_ref(target: bindings::PVIGEM_TARGET) -> Self {
        Self { target, is_ref: true, client: None }
    }

    pub fn update(&self, report: &UsbReport) -> Result<()> {
        let client = self.client.ok_or(Error::TargetUninitialized)?;
        unsafe { Error::new(bindings::vigem_target_x360_update(client, self.target, report.to_raw())).into_result() }
    }

    pub fn register_notification<'a, F: 'a + FnMut(&Target, u8, u8)>(
        &mut self,
        func: F,
    ) -> Result<Pin<Box<NotificationHandle<'a>>>> {
        let client = self.client.ok_or(Error::TargetUninitialized)?;
        unsafe {
            let func: Box<dyn FnMut(&Target, u8, u8)> = Box::new(func);
            let func: Box<dyn FnMut(&Target, u8, u8) + 'static> = std::mem::transmute(func);
            let mut handle = Pin::new(Box::new(NotificationHandle { target: self.target, func }));
            Error::new(bindings::vigem_target_x360_register_notification(
                client,
                self.target,
                Some(handle_notification),
                handle.deref_mut() as *mut NotificationHandle as _,
            ))
            .into_result()?;
            Ok(handle)
        }
    }

    pub fn index(&self) -> u32 {
        unsafe { bindings::vigem_target_get_index(self.target) }
    }

    pub fn user_index(&self) -> Result<u8> {
        let mut index = 0;
        let client = self.client.ok_or(Error::TargetUninitialized)?;
        unsafe {
            Error::new(bindings::vigem_target_x360_get_user_index(client, self.target, &mut index)).into_result()?;
        }
        Ok(index as u8)
    }
}

impl Drop for Target {
    fn drop(&mut self) {
        unsafe {
            if !self.is_ref {
                if let Some(client) = self.client.take() {
                    bindings::vigem_target_remove(client, self.target);
                }
                bindings::vigem_target_free(self.target);
            }
        }
    }
}

impl Drop for NotificationHandle<'_> {
    fn drop(&mut self) {
        unsafe {
            bindings::vigem_target_x360_unregister_notification(self.target);
        }
    }
}
