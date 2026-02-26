use anyhow::Result;
use rusb::{Context, UsbContext};

const QINHENG_VID: u16 = 0x1a86;
const QINHENG_PID: u16 = 0xfe07;

struct LightStripController {
    handle: rusb::DeviceHandle<Context>,
}

impl LightStripController {
    fn new() -> Result<Self> {
        let context = Context::new()?;
        let device = context
            .devices()?
            .iter()
            .find(|dev| {
                dev.device_descriptor()
                    .map(|desc| desc.vendor_id() == QINHENG_VID && desc.product_id() == QINHENG_PID)
                    .unwrap_or(false)
            })
            .ok_or_else(|| anyhow::anyhow!("Device not found (VID:PID {:04x}:{:04x})", QINHENG_VID, QINHENG_PID))?;

        let handle = device.open()?;

        // Detach kernel driver if active
        for iface in 0..2 {
            if let Ok(active) = handle.kernel_driver_active(iface) {
                if active {
                    handle.detach_kernel_driver(iface)?;
                }
            }
            handle.claim_interface(iface)?;
        }

        Ok(Self { handle })
    }

    fn calculate_checksum(packet: &[u8]) -> u8 {
        packet.iter().take(packet.len() - 1).fold(0u8, |acc, &x| acc.wrapping_add(x))
    }

    fn send_raw_packet(&self, packet: &[u8; 20]) -> Result<()> {
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
            std::time::Duration::from_millis(100),
        )?;

        self.handle.write_interrupt(0x01, packet, std::time::Duration::from_millis(100))?;

        Ok(())
    }

    fn scan(&self) -> Result<()> {
        println!("Starting packet byte scan (indices 5 to 18)...");
        println!("I will set one byte to 255 at a time and wait 1 second.");
        println!("Watch the strip and note which index produces Green or Blue.");
        
        for i in 5..=18 {
            if i == 11 || i == 15 { continue; } // Skip brightness
            
            println!("Testing byte index: {}", i);
            let mut packet = [0u8; 20];
            packet[0] = 0x52; packet[1] = 0x42; packet[2] = 20; packet[3] = 1;
            packet[4] = 0x86;
            packet[5] = 0; packet[6] = 255;
            packet[11] = 255; // Max brightness
            packet[15] = 255;
            
            packet[i] = 255;
            packet[19] = Self::calculate_checksum(&packet);
            
            self.send_raw_packet(&packet)?;
            std::thread::sleep(std::time::Duration::from_millis(1000));
        }
        
        println!("Scan complete.");
        Ok(())
    }

    fn send_color(&self, r: u8, g: u8, b: u8, master: u8) -> Result<()> {
        let mut packet = [0u8; 20];

        packet[0] = 0x52;  // 'R'
        packet[1] = 0x42;  // 'B'
        packet[2] = 20;    // length
        packet[3] = 1;     // device ID
        packet[4] = 0x86;  // action: setSectionLED
        packet[5] = 0;     // section start
        packet[6] = 255;   // section end
        packet[7] = 0;
        packet[8] = 0;
        packet[9] = 0;
        packet[10] = 0;
        
        // The True Protocol: 11=R, 12=G, 13=B, 14=Master
        packet[11] = r;
        packet[12] = g;
        packet[13] = b;
        packet[14] = master;
        
        packet[15] = r;
        packet[16] = g;
        packet[17] = b;
        packet[18] = master;
        packet[19] = Self::calculate_checksum(&packet);

        self.send_raw_packet(&packet)
    }

    fn off(&self) -> Result<()> {
        self.send_color(0, 0, 0, 0)
    }

    fn rainbow_cycle(&self, duration_ms: u64) -> Result<()> {
        let colors = vec![
            (255, 0, 0),
            (255, 127, 0),
            (255, 255, 0),
            (0, 255, 0),
            (0, 0, 255),
            (75, 0, 130),
            (148, 0, 211),
        ];

        for (r, g, b) in colors {
            self.send_color(r, g, b, 255)?;
            std::thread::sleep(std::time::Duration::from_millis(duration_ms));
        }

        Ok(())
    }
}

fn print_usage(program_name: &str) {
    println!("Usage:");
    println!("  {} <r> <g> <b> <master>     - Set RGB and Master Brightness (0-255)", program_name);
    println!("  {} scan                     - Scan packet bytes", program_name);
    println!("  {} off                      - Turn off", program_name);
    println!("  {} rainbow <ms>             - Rainbow cycle", program_name);
    println!("\nExamples:");
    println!("  {} 255 0 0 255              # Pure Bright Red", program_name);
    println!("  {} 255 0 255 255            # Purple (Red + Blue)", program_name);
    println!("  {} 0 255 255 255            # Cyan (Green + Blue)", program_name);
}

fn main() -> Result<()> {
    let args: Vec<String> = std::env::args().collect();

    if args.len() < 2 {
        print_usage(&args[0]);
        return Ok(());
    }

    let controller = match LightStripController::new() {
        Ok(c) => c,
        Err(e) => {
            eprintln!("Error: {}", e);
            eprintln!("\nMake sure to run with sudo:");
            eprintln!("  sudo {} <args>", args[0]);
            return Err(e);
        }
    };

    match args[1].as_str() {
        "scan" => {
            controller.scan()?;
        }
        "off" => {
            controller.off()?;
            println!("Light strip turned off");
        }
        "rainbow" => {
            let duration = args
                .get(2)
                .and_then(|s| s.parse().ok())
                .unwrap_or(1000);
            println!("Running rainbow cycle ({}ms per color)...", duration);
            controller.rainbow_cycle(duration)?;
            println!("Rainbow cycle complete");
        }
        _ => {
            let r: u8 = args.get(1).and_then(|s| s.parse().ok()).unwrap_or(0);
            let g: u8 = args.get(2).and_then(|s| s.parse().ok()).unwrap_or(0);
            let b: u8 = args.get(3).and_then(|s| s.parse().ok()).unwrap_or(0);
            let master: u8 = args.get(4).and_then(|s| s.parse().ok()).unwrap_or(255);

            controller.send_color(r, g, b, master)?;
            println!("Set RGB({}, {}, {}) with Master Brightness {}", r, g, b, master);
        }
    }

    Ok(())
}
