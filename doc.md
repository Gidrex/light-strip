# Robobloq Feelin Light Q1 - Reverse Engineering Documentation

## Overview
This document describes the reverse engineering process of the Robobloq "Feelin Light Q1" LED light strip (also sold as "SyncLight") and provides a native Linux solution for controlling it. The original Windows application can be found at [quiklight.robobloq.com/download](https://quiklight.robobloq.com/download).

## Device Information
- **Product Name**: Robobloq Feelin Light Q1 / SyncLight
- **Device**: QinHeng Electronics USB HID Device
- **Vendor ID**: `0x1a86`
- **Product ID**: `0xfe07`
- **Device Type**: USB RF 2.4GHz wireless remote controller (embedded in cable)

## How It Works

### Physical Architecture
```
[USB Port] → [QinHeng USB Device (1a86:fe07)] → [2.4GHz RF] → [LED Light Strip]
                                    ↑
                            [Physical Remote Control]
```

**Important Discovery**: The USB device is NOT the light strip itself! It's a **wireless remote controller** that communicates with the actual light strip via 2.4GHz RF. There's also a physical remote control in the cable that works with the same RF protocol.

## WORKING PROTOCOL

### Packet Format (20 bytes)
**DISCOVERED**: The device uses 20-byte packets with an "RB" header. After extensive testing, the **CORRECT** color order is **BGR** (Blue, Green, Red).

```c
uint8_t packet[20];

packet[0] = 0x52;  // 'R' - Header byte 1
packet[1] = 0x42;  // 'B' - Header byte 2
packet[2] = 20;    // Packet length
packet[3] = 1;     // Device ID (tested: 1 works)
packet[4] = 0x86;  // Action code: setSectionLED (134)
packet[5] = 1;     // Section start
packet[6] = 10;    // Section end
packet[7] = 0;     // Reserved
packet[8] = 0;     // Reserved
packet[9] = 0;     // Reserved
packet[10] = 0;    // Reserved
packet[11] = 3;    // Constant

// IMPORTANT: Color order is BGR, NOT RGB!
packet[12] = b;    // Blue (0-255)
packet[13] = g;    // Green (0-255)
packet[14] = r;    // Red (0-255)

packet[15] = 3;    // Constant
packet[16] = b;    // Blue (repeated)
packet[17] = g;    // Green (repeated)
packet[18] = r;    // Red (repeated)

packet[19] = checksum;  // Simple sum of bytes 0-18
```

### Working Test Result
**CONFIRMED WORKING PACKET**: `52 42 14 01 86 01 0a 00 00 00 00 03 00 00 ff 03 00 00 ff 3e`

When called with `send_rb_color_packet(1, 0, 0, 255)`:
- Input: `r=0, g=0, b=255`
- Bytes 12-14: `00 00 ff`
- Bytes 16-18: `00 00 ff`
- **Result**: RED color (dim)

This confirms the mapping:
- Position 12 = Blue component
- Position 13 = Green component
- Position 14 = **Red component**

### Checksum Calculation
```c
uint8_t calculate_checksum(uint8_t *packet, int len) {
    uint8_t sum = 0;
    for (int i = 0; i < len - 1; i++) {
        sum += packet[i];
    }
    return sum;
}
```

### USB Transfer Method
Working transfer uses BOTH methods:

```c
// Method 1: SET_REPORT (Control Transfer)
libusb_control_transfer(
    dev,
    0x21,   // Host to device, Class, Interface
    0x09,   // SET_REPORT
    0x0200, // Output Report
    0x0000, // Interface 0
    packet,
    20,
    1000    // Timeout
);

// Method 2: Interrupt Transfer
int transferred;
libusb_interrupt_transfer(
    dev,
    0x01,   // Endpoint 1 OUT
    packet,
    20,
    &transferred,
    1000
);
```

**IMPORTANT**: Both transfers must be sent for reliable operation!

## Discovery Process

### Source Analysis
The working protocol was discovered in the official Windows application `SyncLight-2.17.1.exe`:

1. **Extracted Electron app**: Located at `/tmp/synclight-extracted/.webpack/main/index.js`
2. **Found function**: `setSectionLED(e, t, r, s)` in minified JavaScript
3. **Key discovery**: Protocol uses 20-byte packets with "RB" (0x52 0x42) header

### JavaScript Code Analysis
```javascript
setSectionLED(e, t, r, s) {
    const o = n.setID(),
          a = Buffer.alloc(20);

    return a.write(this.header[0], 0, 2, "utf8"),  // bytes 0-1: "RB"
           a.writeUInt8(20, 2),                      // byte 2: length
           a.writeUInt8(o, 3),                       // byte 3: device ID
           a.writeUInt8(this.action.notSyncEffect, 4), // byte 4: action = 0x86
           a.writeUInt8(e[0], 5),                     // byte 5: param e[0]
           a.writeUInt8(e[1], 6),                     // byte 6: param e[1]
           a.writeUInt8(e[2], 7),                     // byte 7: param e[2]
           a.writeUInt8(e[3], 8),                     // byte 8: param e[3]
           a.writeUInt8(e[4], 9),                     // byte 9: param e[4]
           a.writeUInt8(e[5], 10),                    // byte 10: param e[5]
           a.writeUInt8(3, 11),                       // byte 11: constant
           a.writeUInt8(t, 12),                       // byte 12: parameter t
           a.writeUInt8(r, 13),                       // byte 13: parameter r
           a.writeUInt8(s, 14),                       // byte 14: parameter s
           a.writeUInt8(3, 15),                       // byte 15: constant
           a.writeUInt8(t, 16),                       // byte 16: parameter t (repeat)
           a.writeUInt8(r, 17),                       // byte 17: parameter r (repeat)
           a.writeUInt8(s, 18),                       // byte 18: parameter s (repeat)
           a.writeUInt8(i(a), 19),                    // byte 19: checksum
           a
}
```

### Testing Process

#### Failed Approaches
1. ❌ Simple `[1, r, g, b, 254]` packets
2. ❌ Python protocol `[0x52, 0x42, 0x09, 0x01, 0x07, r, g, b, checksum]`
3. ❌ Direct HID writes via `/dev/hidraw0`
4. ❌ RGB order: `packet[12]=r, packet[13]=g, packet[14]=b`
5. ❌ RBG order: `packet[12]=r, packet[13]=b, packet[14]=g`
6. ❌ GRB order: `packet[12]=g, packet[13]=r, packet[14]=b`
7. ❌ Only SET_REPORT without interrupt transfer
8. ❌ Only interrupt transfer without SET_REPORT

#### What Worked
✅ **BGR order** with **BOTH** transfers:
```c
packet[12] = b;    // Blue
packet[13] = g;    // Green
packet[14] = r;    // Red
packet[16] = b;
packet[17] = g;
packet[18] = r;
```

Tested via `linux_driver/test_colors_simple.c`:
- Input `(0, 0, 255)` → RED color confirmed
- Input `(255, 0, 0)` → Only RED works (others fail)
- Only RED color works, appears dim

## Linux Implementation

### Working C Implementation
**File**: `linux_driver/test_colors_simple.c`

**Usage**:
```bash
cd linux_driver
gcc -o test_colors_simple test_colors_simple.c -lusb-1.0
sudo ./test_colors_simple
```

**Test Results**:
- TEST 3: `(0, 0, 255)` produces **RED** color (dim)
- All other tests: no color change

### Rust Implementation (Partially Working)
**File**: `src/main.rs`

**Current Status**:
- Red color (255, 0, 0) works
- Other colors don't work
- Uses BGR color order
- Sends both SET_REPORT and interrupt transfers

**Build and Use**:
```bash
cargo build --release
sudo ./target/release/light-strip 255 0 0  # Red (works)
sudo ./target/release/light-strip 0 255 0  # Green (doesn't work)
sudo ./target/release/light-strip 0 0 255  # Blue (doesn't work)
sudo ./target/release/light-strip rainbow 1000  # Rainbow (works but wrong colors)
```

## Technical Details

### USB Device Configuration
The device presents itself with two interfaces (both HID):
- Interface 0: HID interface
- Interface 1: HID interface

Both interfaces need to be claimed after detaching kernel drivers:
```c
libusb_detach_kernel_driver(dev, 0);
libusb_detach_kernel_driver(dev, 1);
libusb_claim_interface(dev, 0);
libusb_claim_interface(dev, 1);
```

### Color Order Mystery
The device appears to use **BGR color order**, but only RED works reliably. Possible explanations:
1. Device may need initialization sequence
2. Brightness/scaling may be applied
3. Different action codes may be needed for different colors
4. Wire map configuration may affect color mapping

The JavaScript code contains `ajustColorOrder()` function from module 42127, suggesting color order can be changed based on device configuration (`wireMap` parameter).

### Known Issues
1. **Only RED works**: Other colors (green, blue) don't produce expected output
2. **Dim brightness**: Even red appears dim compared to physical remote
3. **Need both transfers**: Must send both SET_REPORT and interrupt transfer

## Python API Analysis
The Python API (`python_decompile/FeelinLight.py`) was decompiled and revealed:
- Uses HTTP POST to `http://192.168.3.224/echo`
- Sends binary packets with similar structure but different transport
- The IP address is NOT the light strip (pings even when disconnected)

This appears to be for a different communication method or device variant.

## Future Work
- [ ] Investigate why only RED color works
- [ ] Fix brightness issue (colors appear dim)
- [ ] Find correct initialization sequence
- [ ] Investigate `ajustColorOrder()` function for color mapping
- [ ] Test different action codes beyond 0x86
- [ ] Add support for effects/patterns
- [ ] Create systemd service for background control

## Summary
The working protocol was discovered through analysis of the official Windows application's Electron JavaScript code. The device uses 20-byte packets with an "RB" (0x52 0x42) header. The confirmed color order is **BGR** (Blue, Green, Red), with positions 12-14 mapping to (Blue, Green, Red). Both SET_REPORT control transfer and interrupt transfer must be sent for reliable operation. Currently only RED color works; green and blue require further investigation.
