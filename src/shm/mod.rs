use anyhow::Result;
use memmap2::MmapMut;
use std::fs::OpenOptions;
use std::path::Path;

pub const SHM_SIZE: usize = 77 * 4; // 308 bytes

pub struct ShmListener {
  mmap: MmapMut,
}

impl ShmListener {
  pub fn new<P: AsRef<Path>>(path: P) -> Result<Self> {
    let file = OpenOptions::new()
      .read(true)
      .write(true)
      .create(true)
      .truncate(false)
      .open(path)?;

    file.set_len(SHM_SIZE as u64)?;

    let mmap = unsafe { MmapMut::map_mut(&file)? };

    Ok(Self { mmap })
  }

  pub fn read_leds(&self) -> [[u8; 3]; 77] {
    let mut leds = [[0u8; 3]; 77];
    for (i, led) in leds.iter_mut().enumerate() {
      let offset = i * 4;
      led[0] = self.mmap[offset]; // R
      led[1] = self.mmap[offset + 1]; // G
      led[2] = self.mmap[offset + 2]; // B
      // Byte 4 is reserved/brightness, ignoring for now or could be used as master
    }
    leds
  }
}
