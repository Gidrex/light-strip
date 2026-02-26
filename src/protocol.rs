use anyhow::{Result, anyhow};
use rusb::{Context, DeviceHandle, UsbContext};
use std::time::Duration;

pub const QINHENG_VID: u16 = 0x1a86;
pub const QINHENG_PID: u16 = 0xfe07;

pub struct Protocol {
    handle: DeviceHandle<Context>,
}

impl Protocol {
    pub fn new() -> Result<Self> {
        let context = Context::new()?;
        let device = context
            .devices()?
            .iter()
            .find(|dev| {
                dev.device_descriptor()
                    .map(|desc| desc.vendor_id() == QINHENG_VID && desc.product_id() == QINHENG_PID)
                    .unwrap_or(false)
            })
            .ok_or_else(|| anyhow!("Device not found (VID:PID {:04x}:{:04x})", QINHENG_VID, QINHENG_PID))?;

        let handle = device.open()?;

        for iface in 0..2 {
            if let Ok(true) = handle.kernel_driver_active(iface) {
                handle.detach_kernel_driver(iface)?;
            }
            handle.claim_interface(iface)?;
        }

        Ok(Self { handle })
    }

    pub fn calculate_checksum(packet: &[u8]) -> u8 {
        packet.iter().take(packet.len() - 1).fold(0u8, |acc, &x| acc.wrapping_add(x))
    }

    pub fn send_raw_packet(&self, packet: &[u8; 20]) -> Result<()> {
        let request_type = rusb::request_type(
            rusb::Direction::Out,
            rusb::RequestType::Class,
            rusb::Recipient::Interface,
        );

        self.handle.write_control(
            request_type,
            0x09,       // bRequest: SET_REPORT
            0x0200u16,  // wValue: Output Report
            0x0000u16,  // wIndex: Interface 0
            packet,
            Duration::from_millis(100),
        )?;

        self.handle.write_interrupt(0x01, packet, Duration::from_millis(100))?;

        Ok(())
    }

    pub fn send_segment(&self, r: u8, g: u8, b: u8, end: u8) -> Result<()> {
        let mut packet = [0u8; 20];
        packet[0] = 0x52; packet[1] = 0x42; packet[2] = 20; packet[3] = 1;
        packet[4] = 0x86;
        packet[5] = 0; packet[6] = 255;
        
        packet[11] = r;
        packet[12] = g;
        packet[13] = b;
        packet[14] = end;
        
        packet[15] = r; packet[16] = g; packet[17] = b; packet[18] = end;
        
        packet[19] = Self::calculate_checksum(&packet);
        self.send_raw_packet(&packet)
    }
}
