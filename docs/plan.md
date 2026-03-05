# Project Engineering Design Document: EmulStick Desktop (Tauri)

## 1. Project Overview

This project aims to rebuild the original EmulStick controller into a high-performance, cross-platform desktop application. The core objective is to provide low-latency, hardware-level keyboard/mouse emulation, support global interception of system shortcuts (e.g., `Alt+Tab`, `Win` key), and allow users to customize passthrough modes.

## 2. Core Technology Stack

| Domain | Technology/Library | Reason for Selection |
| --- | --- | --- |
| **Underlying Framework** | Tauri (Rust) | Extremely low memory footprint and a tiny bundle size (< 10MB). |
| **Frontend UI** | Svelte + Vite + TS | No Virtual DOM, compiles to pure JS, ensuring a lightweight interface and incredibly fast startup. |
| **Bluetooth Comms** | `btleplug` (Rust) | Provides cross-platform (Win/Mac/Linux) asynchronous BLE device scanning and communication capabilities. |
| **Input Interception** | `rdev` (Rust) | Cross-platform OS-level global hooks. Can intercept system shortcuts and mouse trajectories, breaking through browser limitations. |
| **Video Capture** | WebRTC API (Frontend) | Directly utilizes HTML5 `<video>` and `getUserMedia` to read the HDMI capture card feed, delivering excellent performance. |

## 3. System Architecture and Data Flow

The system is divided into three major modules, adopting a "control plane (frontend)" and "data plane (backend)" separation design to ensure maximum performance.

### Architecture Concept

* **Frontend (Svelte)**: Responsible for triggering device scanning, displaying connection status, toggling passthrough modes, and rendering the HDMI capture feed.
* **Tauri IPC Channel**: Only transmits "control commands" (e.g., connect to Device A, enable keyboard passthrough, disable mouse passthrough), and **does not** transmit high-frequency cursor coordinates.
* **Backend (Rust)**: Runs a BLE connection pool in the background; upon receiving a "start control" command, it launches `rdev` to listen for OS-level hardware interrupt events and directly translates them into EmulStick BLE payloads to write to the hardware.

## 4. Core Functional Requirements

### 4.1 Device Connection Management (BLE)

* Provide a UI for users to scan and list nearby EmulStick devices.
* Remember the MAC address of the last connected device, supporting automatic reconnection upon startup.

### 4.2 OS-Level Input Hooking

* **State Trigger**: Users click on the screen or press a specific hotkey (e.g., `Ctrl+Alt`) to enter/exit "Control Lock Mode."
* **Keystroke Interception**: Once in lock mode, the application intercepts all keyboard inputs, including system reserved keys (Windows key, Command key, Alt+Tab, etc.). No response is generated locally; all inputs are converted and transmitted to the target host.
* **Mouse Locking**: Locks the cursor to the center of the window, capturing relative movement (Delta X/Y) and scroll wheel events.

### 4.3 Passthrough Control

Provides three independent boolean states that can be toggled in real-time:

* `enable_keyboard`: Whether to intercept and send keyboard events.
* `enable_mouse`: Whether to intercept and send mouse/trackpad events.
* `enable_video`: Whether to activate and render the HDMI capture card video stream.
*(For example: The user can check only "Keyboard"; in this case, the local mouse can still operate local windows normally, but keyboard typing will be sent to the controlled host.)*

### 4.4 Video Integration & Display

* Detect available UVC (USB Video Class) camera devices on the system.
* Allow users to select an HDMI capture card from a dropdown menu and set the resolution (e.g., 1080p@60fps) to output as the main window background.

## 5. Permissions & Security

To implement global system interception, different operating systems require specific permissions handling:

* **Windows**: No special restrictions, though some antivirus software might issue warnings regarding global hooking.
* **macOS**: Requires declaring Bluetooth permissions (`NSBluetoothAlwaysUsageDescription`) in `Info.plist`. More importantly, on first launch, it must guide the user to grant the application permissions via "System Settings -> Privacy & Security -> Accessibility", otherwise `rdev` cannot intercept system keys.
* **Linux (Wayland/X11)**: Global hook support is generally poorer in Wayland environments. It is necessary to ensure the program runs under X11 or handle inputs via `uinput`.

## 6. Payload Protocol Definition

After Rust intercepts a keystroke or mouse event, it must be translated into the byte array specified by the EmulStick hardware.
*(The following is illustrative and should refer to the original project's packet format)*

* **Mouse Payload**: `[Report ID, Buttons, X (Low), X (High), Y (Low), Y (High), Wheel, H-Wheel]`
* **Keyboard Payload**: `[Report ID, Modifiers, Reserved, Key1, Key2, Key3, Key4, Key5, Key6]`