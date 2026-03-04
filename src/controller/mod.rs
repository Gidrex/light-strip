use crate::protocol::Protocol;
use anyhow::Result;

pub const NUM_LEDS: usize = 77;

pub struct Canvas {
  pub leds: [[u8; 3]; NUM_LEDS],
}

impl Canvas {
  pub fn new() -> Self {
    Self {
      leds: [[0; 3]; NUM_LEDS],
    }
  }

  #[allow(dead_code)]
  pub fn set_led(&mut self, index: usize, r: u8, g: u8, b: u8) {
    if index < NUM_LEDS {
      self.leds[index] = [r, g, b];
    }
  }

  #[allow(dead_code)]
  pub fn clear(&mut self) {
    self.leds = [[0; 3]; NUM_LEDS];
  }
}

pub struct RenderEngine {
  protocol: Protocol,
}

impl RenderEngine {
  pub fn new(protocol: Protocol) -> Self {
    Self { protocol }
  }

  /// Renders the canvas to the physical strip using the Painter's Algorithm.
  /// To support individual LEDs, we send segments from 0..77 in descending order.
  pub fn flush(&self, canvas: &Canvas) -> Result<()> {
    // Simplified approach for now:
    // 1. Find all points where color changes.
    // 2. Send segments from 0 to that point, in descending order of the point.

    let mut change_points = Vec::new();
    let mut prev_color = None;
    for i in 0..NUM_LEDS {
      let color = canvas.leds[i];
      if Some(color) != prev_color {
        change_points.push(i);
        prev_color = Some(color);
      }
    }
    // Add the very end
    change_points.push(NUM_LEDS);

    // Send in descending order of index
    for &idx in change_points.iter().rev() {
      if idx == 0 {
        continue;
      }
      let color = canvas.leds[idx - 1];
      self
        .protocol
        .send_segment(color[0], color[1], color[2], idx as u8)?;
      // Small delay to let the controller process
      std::thread::sleep(std::time::Duration::from_millis(5));
    }

    Ok(())
  }
}
