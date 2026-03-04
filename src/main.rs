mod controller;
mod effects;
mod protocol;
mod shm;

use crate::controller::{Canvas, RenderEngine};
use crate::protocol::Protocol;
use crate::shm::ShmListener;
use anyhow::Result;
use std::time::{Duration, Instant};

fn print_usage(program_name: &str) {
  println!("Usage:");
  println!(
    "  {} static <r> <g> <b> <m>  - Set static color and brightness",
    program_name
  );
  println!(
    "  {} rainbow                 - Run rainbow effect (foreground)",
    program_name
  );
  println!(
    "  {} wave                    - Run wave effect (foreground)",
    program_name
  );
  println!(
    "  {} shm                     - Read from /tmp/feelinlight.bin (foreground)",
    program_name
  );
  println!("  {} off                     - Turn off", program_name);
  println!("\nExamples:");
  println!(
    "  {} static 245 245 220 255  # Pleasant Beige",
    program_name
  );
}

#[tokio::main]
async fn main() -> Result<()> {
  let args: Vec<String> = std::env::args().collect();
  if args.len() < 2 {
    print_usage(&args[0]);
    return Ok(());
  }

  let protocol = Protocol::new()?;

  match args[1].as_str() {
    "static" => {
      let r: u8 = args.get(2).and_then(|s| s.parse().ok()).unwrap_or(0);
      let g: u8 = args.get(3).and_then(|s| s.parse().ok()).unwrap_or(0);
      let b: u8 = args.get(4).and_then(|s| s.parse().ok()).unwrap_or(0);
      let end: u8 = args.get(5).and_then(|s| s.parse().ok()).unwrap_or(255);

      protocol.send_segment(r, g, b, end)?;
      println!(
        "Set static color RGB({}, {}, {}) up to index {}",
        r, g, b, end
      );
    }
    "off" => {
      protocol.send_segment(0, 0, 0, 77)?;
      println!("Turned off");
    }
    "rainbow" | "wave" | "shm" => {
      let mode = args[1].as_str();
      println!("Running {} mode. Press Ctrl+C to stop.", mode);

      let engine = RenderEngine::new(protocol);
      let mut canvas = Canvas::new();

      let shm = if mode == "shm" {
        Some(ShmListener::new("/tmp/feelinlight.bin")?)
      } else {
        None
      };

      let start_time = Instant::now();
      let mut last_render = Instant::now();
      let frame_duration = Duration::from_millis(33); // 30 FPS

      loop {
        let now = Instant::now();
        let t = start_time.elapsed().as_secs_f32();

        match mode {
          "rainbow" => effects::rainbow(t * 50.0, &mut canvas.leds),
          "wave" => effects::wave(t * 2.0, &mut canvas.leds),
          "shm" => {
            if let Some(ref listener) = shm {
              canvas.leds = listener.read_leds();
            }
          }
          _ => {}
        }

        if now.duration_since(last_render) >= frame_duration {
          engine.flush(&canvas)?;
          last_render = now;
        }

        tokio::time::sleep(Duration::from_millis(5)).await;
      }
    }
    _ => {
      print_usage(&args[0]);
    }
  }

  Ok(())
}
