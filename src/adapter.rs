use rusb::constants::LIBUSB_DT_HID;
use rusb::{DeviceHandle, UsbContext};
use std::convert::TryInto;
use std::time::Duration;

#[derive(Clone, Copy)]
pub struct GCPad {
    pub buttons: u16,
    pub stick_x: u8,
    pub stick_y: u8,
    pub cstick_x: u8,
    pub cstick_y: u8,
    pub trigger_left: u8,
    pub trigger_right: u8,
}

pub struct GCAdapterWaiter {
    context: rusb::Context,
}

impl GCAdapterWaiter {
    pub fn new() -> rusb::Result<Self> {
        Ok(Self {
            context: rusb::Context::new()?,
        })
    }

    pub fn try_connect_controller(&self) -> rusb::Result<Option<GCAdapter>> {
        if let Some(device) = self.context.devices()?.iter().find(|device| {
            device
                .device_descriptor()
                .map(|desc| desc.vendor_id() == 0x057e && desc.product_id() == 0x0337)
                .unwrap_or(false)
        }) {
            let mut handle = device.open()?;
            match handle.kernel_driver_active(0) {
                Ok(true) => handle.detach_kernel_driver(0)?,
                Ok(false) => (),
                Err(rusb::Error::NotSupported) => (),
                Err(e) => Err(e)?,
            }

            // nyko
            match handle.write_control(0x21, 11, 0x0001, 0, &[], Duration::from_secs(1)) {
                Ok(_) | Err(rusb::Error::Pipe) => (), // mayflash
                Err(e) => return Err(e),
            }

            handle.claim_interface(0)?;

            let mut endpoint_in = 0;
            let mut endpoint_out = 0;
            for int in device.config_descriptor(0).unwrap().interfaces() {
                for desc in int.descriptors() {
                    for ep in desc.endpoint_descriptors() {
                        match ep.direction() {
                            rusb::Direction::In => endpoint_in = ep.address(), // save address
                            rusb::Direction::Out => endpoint_out = ep.address(), // save address
                        }
                    }
                }
            }

            handle.write_interrupt(endpoint_out, &[0x13], Duration::from_millis(16))?;

            Ok(Some(GCAdapter {
                handle,
                endpoint_in,
                endpoint_out,
            }))
        } else {
            Ok(None)
        }
    }
}

pub struct GCAdapter {
    handle: DeviceHandle<rusb::Context>,
    endpoint_in: u8,
    endpoint_out: u8,
}

impl GCAdapter {
    pub fn get_pads(&self) -> [Option<GCPad>; 4] {
        let mut payload = [0; 37];
        if self
            .handle
            .read_interrupt(self.endpoint_in, &mut payload, Duration::from_millis(16))
            .unwrap()
            != 37
            || payload[0] != LIBUSB_DT_HID
        {
            // might happen a few times on init
            return Default::default();
        }

        let mut output = [None; 4];
        for (i, chunk) in payload[1..].chunks_exact(9).enumerate() {
            if chunk[0] != 0 {
                output[i] = Some(GCPad {
                    buttons: u16::from_le_bytes(chunk[1..3].try_into().unwrap()),
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

        output
    }
}
