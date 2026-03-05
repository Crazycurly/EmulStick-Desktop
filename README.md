# EmulStick-Desktop

A cross-platform desktop application that acts as a BLE (Bluetooth Low Energy) KVM (Keyboard-Video-Mouse) controller. Connect to EmulStick hardware devices to control remote systems with low-latency keyboard and mouse emulation.

[![Build](https://github.com/EmulStick/EmulStick-Desktop/actions/workflows/build.yml/badge.svg)](https://github.com/EmulStick/EmulStick-Desktop/actions/workflows/build.yml)

## Features

- **BLE Device Connection** - Scan and connect to EmulStick hardware devices
- **Global Input Interception** - Capture keyboard and mouse events at the OS level, including system shortcuts (Alt+Tab, Windows key, etc.)
- **Lock Mode** - Toggle control mode to intercept all input and send to the remote device
- **Passthrough Controls** - Independently enable/disable keyboard, mouse, and video passthrough
- **HDMI Capture Video** - Display video feed from HDMI capture cards via WebRTC

## Technology Stack

| Component | Technology |
|-----------|------------|
| Desktop Framework | [Tauri](https://tauri.app/) (Rust) |
| Frontend UI | [Svelte](https://svelte.dev/) + TypeScript |
| BLE Communication | [btleplug](https://github.com/deviceplug/btleplug) |
| Input Hooks | [rdev](https://github.com/roderickvd/rdev) |
| Video Capture | WebRTC / getUserMedia API |

## Architecture

EmulStick-Desktop uses a control plane / data plane separation:

- **Frontend (Svelte)**: Handles device scanning UI, connection status display, passthrough toggles, and video feed rendering
- **Backend (Rust/Tauri)**: Manages BLE connections, intercepts OS-level input events, and transmits to hardware
- **IPC Channel**: Transmits control commands (connect, disconnect, enable/disable passthrough) - does not carry high-frequency cursor data

## Building from Source

### Prerequisites

- Node.js 18+
- Rust 1.70+
- System dependencies:

**Ubuntu/Debian:**
```bash
sudo apt-get install libwebkit2gtk-4.1-dev libappindicator3-dev librsvg2-dev patchelf libbluez-dev libudev-dev
```

**macOS:**
- Xcode Command Line Tools
- Bluetooth system permissions

**Windows:**
- Visual Studio Build Tools
- WebView2 Runtime

### Build Commands

```bash
# Install dependencies
npm install

# Development mode
npm run tauri dev

# Build for production
npm run tauri build
```

## Usage

1. **Launch the application**
2. **Scan for devices** - Click the scan button to discover nearby EmulStick devices
3. **Connect** - Select a device from the list to connect
4. **Enter Lock Mode** - Press `Ctrl+Alt` to toggle lock mode and start controlling the remote system
5. **Configure Passthrough** - Use the toggles to enable/disable keyboard, mouse, and video passthrough independently

## BLE Protocol

The BLE protocol specification is documented in [docs/ble-protocol.md](docs/ble-protocol.md).

### Service UUIDs

| Type | UUID |
|------|------|
| Custom Service | `0000F800-0000-1000-8000-00805f9b34fb` |
| Keyboard Characteristic | `0000F801-0000-1000-8000-00805f9b34fb` |
| Mouse Characteristic | `0000F803-0000-1000-8000-00805f9b34fb` |

## Platform-Specific Notes

### Windows
- No special permissions required beyond standard installation
- Some antivirus software may warn about global hooking

### macOS
- Requires Bluetooth permissions (prompted on first launch)
- Requires Accessibility permissions (System Settings > Privacy & Security > Accessibility) for global input interception
- Grant permissions when prompted for `rdev` to function

### Linux
- X11 recommended for global hook support
- Wayland support is limited; consider running under X11 or via uinput

## License

MIT License