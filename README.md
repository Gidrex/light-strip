# light-strip (Linux QuikLight Replacement)

A high-performance Rust implementation for controlling LED strips, specifically designed as a native Linux replacement for the Windows [QuikLight (RoboBloq)](https://quiklight.robobloq.com/download) application.

## Overview

This project provides a low-level, efficient way to control USB-based LED strips on Linux. It bypasses the need for Wine or Windows virtual machines by communicating directly with the hardware using the rusb library.

### Key Features

- Native Linux Support: Built from the ground up for Linux (Tested on Wayland/Niri).
- Zero Latency: Direct USB HID communication with the QinHeng (VID: 0x1a86, PID: 0xfe07) controller.
- Shared Memory (SHM) Integration: Sync your lights with external tools (like feelinlight) via /tmp/feelinlight.bin for screen-matching or music visualization.
- Built-in Effects: Includes hardware-accelerated Rainbow and Wave effects.
- Precise Control: Support for setting static colors across the entire strip or specific segments.

## How it Works

The strip consists of 77 individually addressable LEDs. Unlike standard serial protocols, this controller uses a "Segment Painting" approach:

1. USB HID Protocol: The program sends 20-byte control packets to the device.
2. Painter's Algorithm: To update the entire strip, the RenderEngine identifies color changes and sends segments in descending order (from the end to the start). This allows for complex gradients and individual LED control despite protocol limitations.
3. Shared Memory Buffer: By mapping a file in /tmp, other applications can write raw RGB data directly to the memory, which light-strip then flushes to the hardware at 30 FPS.

## Installation

Ensure you have rust and libusb headers installed on your system.

```bash
# Clone the repository
git clone https://github.com/gidrex/light-strip
cd light-strip

# Build the project
just release
```

### Permission Setup (udev)

To run without sudo, add a udev rule for the USB device:

```bash
❯ lsusb | rg -I 1a86
Bus 003 Device 014: ID 1a86:fe07 QinHeng Electronics USBHID

# /etc/udev/rules.d/99-light-strip.rules
SUBSYSTEM=="usb", ATTR{idVendor}=="1a86", ATTR{idProduct}="fe07", MODE="0666"
```

## Usage

The project uses Justfile for common tasks.

```bash
# Set a static color (<R> <G> <B> <Sectors to write>)
./target/release/light-strip static 255 100 0 77

# Run the Rainbow effect
./target/release/light-strip rainbow

# Sync with Shared Memory (from /tmp/feelinlight.bin)
./target/release/light-strip shm

# Turn off the lights
./target/release/light-strip off
```

## Project Structure

- src/protocol.rs: Low-level USB HID packet construction and checksum calculation.
- src/controller/: The RenderEngine and Canvas logic for managing LED states.
- src/shm/: Shared memory listener for inter-process communication.
- src/effects/: Math-based animations (Rainbow, Wave).

## Documentation

Detailed reverse engineering process and protocol information can be found in [doc.md](doc.md).

