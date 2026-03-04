pub fn wave(t: f32, leds: &mut [[u8; 3]; 77]) {
  for (i, led) in leds.iter_mut().enumerate() {
    let phase = (i as f32 / 77.0) * 2.0 * std::f32::consts::PI;
    let val = ((t + phase).sin() + 1.0) / 2.0;

    led[0] = (val * 255.0) as u8; // Red wave
    led[1] = ((1.0 - val) * 128.0) as u8; // Green opposite
    led[2] = 128; // Constant blue
  }
}

pub fn rainbow(t: f32, leds: &mut [[u8; 3]; 77]) {
  for (i, led) in leds.iter_mut().enumerate() {
    let hue = (t + (i as f32 / 77.0) * 360.0) % 360.0;
    let (r, g, b) = hsv_to_rgb(hue, 1.0, 1.0);
    *led = [r, g, b];
  }
}

fn hsv_to_rgb(h: f32, s: f32, v: f32) -> (u8, u8, u8) {
  let c = v * s;
  let x = c * (1.0 - ((h / 60.0) % 2.0 - 1.0).abs());
  let m = v - c;
  let (r, g, b) = if h < 60.0 {
    (c, x, 0.0)
  } else if h < 120.0 {
    (x, c, 0.0)
  } else if h < 180.0 {
    (0.0, c, x)
  } else if h < 240.0 {
    (0.0, x, c)
  } else if h < 300.0 {
    (x, 0.0, c)
  } else {
    (c, 0.0, x)
  };
  (
    ((r + m) * 255.0) as u8,
    ((g + m) * 255.0) as u8,
    ((b + m) * 255.0) as u8,
  )
}
