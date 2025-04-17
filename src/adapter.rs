// This is based on Dolphin's code for handling the GC adapter.
// https://github.com/dolphin-emu/dolphin/blob/master/Source/Core/InputCommon/GCAdapter.cpp

use crate::{config::GButton, log, ui};
use native_windows_gui as nwg;
use parking_lot::{Condvar, Mutex, Once};
use rusb::{Context, Device, DeviceHandle, Hotplug, HotplugBuilder, UsbContext, constants::LIBUSB_DT_HID};
use std::{convert::TryInto, sync::Arc, time::Duration};

const ADAPTER_VENDOR_ID: u16 = 0x057e;
const ADAPTER_PRODUCT_ID: u16 = 0x0337;

#[derive(Clone, Copy)]
pub struct GCPad {
    pub buttons: GButton,
    pub stick_x: u8,
    pub stick_y: u8,
    pub cstick_x: u8,
    pub cstick_y: u8,
    pub trigger_left: u8,
    pub trigger_right: u8,
}

pub struct GCAdapterWaiter {
    context: rusb::Context,
    adapter: Arc<(Mutex<Option<GCAdapter>>, Condvar)>,
    hotplug_reg: Option<rusb::Registration<rusb::Context>>,
    newly_none: Arc<Mutex<bool>>,
    exit_once: Arc<Once>,
    logger: ui::Logger,
    exit_sender: nwg::NoticeSender,
}

struct HotplugCallback {
    adapter: Arc<(Mutex<Option<GCAdapter>>, Condvar)>,
}

impl Hotplug<rusb::Context> for HotplugCallback {
    fn device_arrived(&mut self, _device: Device<Context>) {
        self.adapter.1.notify_all();
    }

    fn device_left(&mut self, _device: Device<Context>) {
        *self.adapter.0.lock() = None;
    }
}

impl GCAdapterWaiter {
    pub fn new(exit_once: Arc<Once>, logger: ui::Logger, exit_sender: nwg::NoticeSender) -> rusb::Result<Self> {
        let context = rusb::Context::new()?;
        let adapter = Arc::new((Mutex::new(None), Condvar::new()));
        let hotplug_reg = if rusb::has_hotplug() {
            match HotplugBuilder::new()
                .vendor_id(ADAPTER_PRODUCT_ID)
                .product_id(ADAPTER_PRODUCT_ID)
                .register(&context, Box::new(HotplugCallback { adapter: adapter.clone() }))
            {
                Ok(reg) => {
                    log!(logger, "Using libusb hotplug detection.");
                    Some(reg)
                },
                Err(e) => {
                    log!(logger, "Couldn't initialize libusb hotplug detection: {}", e);
                    None
                },
            }
        } else {
            None
        };
        Ok(Self {
            context,
            adapter,
            hotplug_reg,
            newly_none: Arc::new(Mutex::new(false)),
            exit_once,
            logger,
            exit_sender,
        })
    }

    pub fn try_connect_controller(&self) -> rusb::Result<Option<GCAdapter>> {
        if let Some((device, handle)) = self.context.devices()?.iter().find_map(|device| {
            let descriptor = match device.device_descriptor() {
                Ok(desc) => desc,
                Err(_) => return None,
            };
            if descriptor.vendor_id() == ADAPTER_VENDOR_ID && descriptor.product_id() == ADAPTER_PRODUCT_ID {
                log!(self.logger, "Found GC adapter, opening...");
                let handle = match device.open() {
                    Ok(handle) => handle,
                    Err(rusb::Error::Access) => {
                        log!(
                            self.logger,
                            "ERROR: I don't have access to that device: Bus {:03} Device {:03}: ID {:04X}:{:04X}.\
                            Do you have Dolphin or another copy of this program running?",
                            device.bus_number(),
                            device.port_number(),
                            descriptor.vendor_id(),
                            descriptor.product_id()
                        );
                        return None
                    },
                    Err(rusb::Error::NotSupported) => {
                        ui::show_error(
                            "Driver not found",
                            "Opening the device was not supported.\n\
                            This may mean you haven't installed the WinUSB driver correctly.\n\
                            Please install the driver with Zadig and try again.",
                        );
                        self.exit_sender.notice();
                        return None
                    },
                    Err(e) => {
                        log!(self.logger, "ERROR: couldn't open: {}", e);
                        return None
                    },
                };

                match handle.kernel_driver_active(0) {
                    Ok(true) => {
                        if let Err(e) = handle.detach_kernel_driver(0) {
                            log!(self.logger, "ERROR: couldn't detach kernel driver: {}", e);
                            return None
                        }
                    },
                    Ok(false) => (),
                    Err(rusb::Error::NotSupported) => (),
                    Err(e) => {
                        log!(self.logger, "Error: couldn't check if kernel driver was active: {}", e);
                        return None
                    },
                }

                // nyko
                match handle.write_control(0x21, 11, 0x0001, 0, &[], Duration::from_secs(1)) {
                    Ok(_) | Err(rusb::Error::Pipe) => (), // mayflash
                    Err(e) => {
                        log!(self.logger, "ERROR: Unexpected error in Nyko compat: {}", e);
                        return None
                    },
                }

                match handle.claim_interface(0) {
                    Ok(()) => Some((device, handle)),
                    Err(e) => {
                        log!(self.logger, "ERROR: couldn't claim interface: {}", e);
                        None
                    },
                }
            } else {
                None
            }
        }) {
            let mut endpoint_in = 0;
            let mut endpoint_out = 0;
            for int in device.config_descriptor(0).unwrap().interfaces() {
                for desc in int.descriptors() {
                    for ep in desc.endpoint_descriptors() {
                        match ep.direction() {
                            rusb::Direction::In => endpoint_in = ep.address(),   // save address
                            rusb::Direction::Out => endpoint_out = ep.address(), // save address
                        }
                    }
                }
            }

            handle.write_interrupt(endpoint_out, &[0x13], Duration::from_millis(16))?;

            Ok(Some(GCAdapter { handle, endpoint_in, endpoint_out }))
        } else {
            Ok(None)
        }
    }

    pub fn wait_for_controller(&self) {
        let (lock, cvar) = &*self.adapter;
        let mut adapter_guard = lock.lock();
        if adapter_guard.is_none() {
            log!(self.logger, "Waiting for GC adapter...");
            *self.newly_none.lock() = true;
            *adapter_guard = Some(loop {
                if self.exit_once.state().done() {
                    return
                }
                if let Some(adapter) = self.try_connect_controller().unwrap() {
                    break adapter
                }
                if self.hotplug_reg.is_some() {
                    cvar.wait(&mut adapter_guard);
                } else {
                    std::thread::sleep(Duration::from_millis(500))
                }
            });
            log!(self.logger, "GC adapter connected!");
        }
    }

    pub fn get_pads(&self) -> [Option<GCPad>; 4] {
        let adapter_lock = self.adapter.0.lock();
        let out = if let Some(adapter) = adapter_lock.as_ref() {
            let out = adapter.get_pads(&self.logger);
            drop(adapter_lock);
            out
        } else {
            drop(adapter_lock);
            let mut newly_none = self.newly_none.lock();
            if *newly_none {
                log!(self.logger, "GC adapter disconnected.");
                *newly_none = false;
                return Default::default()
            } else {
                drop(newly_none);
                self.wait_for_controller();
                self.adapter.0.lock().as_ref().map(|ad| ad.get_pads(&self.logger)).unwrap_or_default()
            }
        };
        if let Some(out) = out {
            out
        } else {
            *self.newly_none.lock() = false;
            *self.adapter.0.lock() = None;
            Default::default()
        }
    }

    pub fn send_rumble(&self, rumble: [u8; 4]) {
        // nonblocking
        if let Some(adapter) = self.adapter.0.lock().as_ref() {
            if let Err(e) = adapter.send_rumble(rumble) {
                log!(self.logger, "ERROR: sending rumble failed: {}", e);
            }
        }
    }
}

pub struct GCAdapter {
    handle: DeviceHandle<rusb::Context>,
    endpoint_in: u8,
    endpoint_out: u8,
}

impl GCAdapter {
    pub fn get_pads(&self, logger: &ui::Logger) -> Option<[Option<GCPad>; 4]> {
        let mut payload = [0; 37];
        match self.handle.read_interrupt(self.endpoint_in, &mut payload, Duration::from_millis(16)) {
            Ok(37) if payload[0] == LIBUSB_DT_HID => (),
            Ok(_) => return Some(Default::default()), // might happen a few times on init
            Err(rusb::Error::NoDevice) => {
                log!(logger, "GC adapter disconnected.");
                return None
            },
            Err(rusb::Error::Pipe) => {
                log!(logger, "Endpoint halted, will attempt to reconnect.");
                return None
            },
            Err(e) => {
                log!(logger, "Failed to read from adapter: {}", e);
                return Some(Default::default())
            },
        }

        let mut output = [None; 4];
        for (i, chunk) in payload[1..].chunks_exact(9).enumerate() {
            if chunk[0] >> 4 != 0 {
                output[i] = Some(GCPad {
                    buttons: GButton::from_bits_truncate(u16::from_le_bytes(chunk[1..3].try_into().unwrap())),
                    stick_x: chunk[3],
                    stick_y: chunk[4],
                    cstick_x: chunk[5],
                    cstick_y: chunk[6],
                    trigger_left: chunk[7],
                    trigger_right: chunk[8],
                });
            } else {
                output[i] = None;
            }
        }

        Some(output)
    }

    pub fn send_rumble(&self, rumble: [u8; 4]) -> rusb::Result<()> {
        let payload = [0x11, rumble[0], rumble[1], rumble[2], rumble[3]];
        self.handle.write_interrupt(self.endpoint_out, &payload, Duration::from_millis(16))?;
        Ok(())
    }
}
