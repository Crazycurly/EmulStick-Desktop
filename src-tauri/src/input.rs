use crate::ble::BleState;
use crate::protocol;
use crate::state::SharedAppState;
use rdev::{Button, Event, EventType, Key};
use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::Arc;
use tauri::Emitter;
use tokio::sync::Mutex;

static HOOK_RUNNING: AtomicBool = AtomicBool::new(false);

struct HookContext {
    ble: Arc<Mutex<BleState>>,
    app_state: SharedAppState,
    modifier_state: std::sync::Mutex<u8>,
    button_state: std::sync::Mutex<u8>,
    ctrl_held: AtomicBool,
    alt_held: AtomicBool,
    mouse_x: std::sync::Mutex<f64>,
    mouse_y: std::sync::Mutex<f64>,
    mouse_initialized: AtomicBool,
}

fn is_ctrl(key: Key) -> bool {
    matches!(key, Key::ControlLeft | Key::ControlRight)
}

fn is_alt(key: Key) -> bool {
    matches!(key, Key::Alt | Key::AltGr)
}

/// Map rdev Button to HID mouse button bit.
fn rdev_button_bit(button: Button) -> u8 {
    match button {
        Button::Left => 0x01,
        Button::Right => 0x02,
        Button::Middle => 0x04,
        _ => 0,
    }
}

pub fn start_hook(
    ble: Arc<Mutex<BleState>>,
    app_state: SharedAppState,
    app_handle: tauri::AppHandle,
) -> Result<(), String> {
    if HOOK_RUNNING.swap(true, Ordering::SeqCst) {
        return Err("Hook already running".to_string());
    }

    let ctx = Arc::new(HookContext {
        ble,
        app_state,
        modifier_state: std::sync::Mutex::new(0u8),
        button_state: std::sync::Mutex::new(0u8),
        ctrl_held: AtomicBool::new(false),
        alt_held: AtomicBool::new(false),
        mouse_x: std::sync::Mutex::new(0.0),
        mouse_y: std::sync::Mutex::new(0.0),
        mouse_initialized: AtomicBool::new(false),
    });

    let rt = tauri::async_runtime::handle();

    // --- Keyboard grab thread (rdev::grab) ---
    {
        let ctx = ctx.clone();
        let app_handle = app_handle.clone();
        let rt = rt.clone();

        std::thread::spawn(move || {
            log::info!("Keyboard grab thread started");
            let callback = move |event: Event| -> Option<Event> {
                if !HOOK_RUNNING.load(Ordering::SeqCst) {
                    return Some(event);
                }

                let ctx = ctx.clone();
                let app_handle = app_handle.clone();
                let rt = rt.clone();

                // Track Ctrl/Alt for escape combo
                match event.event_type {
                    // ------ Mouse events ------
                    EventType::MouseMove { x, y } => {
                        let state = ctx.app_state.lock().unwrap();
                        let lock_mode = state.lock_mode;
                        let mouse_enabled = state.passthrough.enable_mouse;
                        drop(state);

                        if lock_mode && mouse_enabled {
                            let (dx, dy) = {
                                let mut mx = ctx.mouse_x.lock().unwrap();
                                let mut my = ctx.mouse_y.lock().unwrap();
                                let initialized = ctx.mouse_initialized.load(Ordering::SeqCst);
                                let (dx, dy) = if initialized {
                                    ((x - *mx) as i16, (y - *my) as i16)
                                } else {
                                    ctx.mouse_initialized.store(true, Ordering::SeqCst);
                                    (0i16, 0i16)
                                };
                                *mx = x;
                                *my = y;
                                (dx, dy)
                            };
                            if dx != 0 || dy != 0 {
                                let dx = dx.clamp(-2047, 2047);
                                let dy = dy.clamp(-2047, 2047);
                                let buttons = *ctx.button_state.lock().unwrap();
                                let payload = protocol::encode_mouse(buttons, dx, dy, 0);
                                let ble = ctx.ble.clone();
                                rt.spawn(async move {
                                    let ble = ble.lock().await;
                                    let _ = ble.write_mouse(payload).await;
                                });
                            }
                            return None; // suppress — cursor stays on host side only
                        }
                        // Update position even when not in lock mode so we don't
                        // get a huge jump when lock mode is first enabled.
                        {
                            let mut mx = ctx.mouse_x.lock().unwrap();
                            let mut my = ctx.mouse_y.lock().unwrap();
                            *mx = x;
                            *my = y;
                            ctx.mouse_initialized.store(true, Ordering::SeqCst);
                        }
                        return Some(event);
                    }
                    EventType::ButtonPress(button) => {
                        let bit = rdev_button_bit(button);
                        if bit != 0 {
                            let mut btns = ctx.button_state.lock().unwrap();
                            *btns |= bit;
                            let buttons = *btns;
                            drop(btns);

                            let state = ctx.app_state.lock().unwrap();
                            let lock_mode = state.lock_mode;
                            let mouse_enabled = state.passthrough.enable_mouse;
                            drop(state);

                            if lock_mode && mouse_enabled {
                                let payload = protocol::encode_mouse(buttons, 0, 0, 0);
                                let ble = ctx.ble.clone();
                                rt.spawn(async move {
                                    let ble = ble.lock().await;
                                    let _ = ble.write_mouse(payload).await;
                                });
                                return None;
                            }
                        }
                        return Some(event);
                    }
                    EventType::ButtonRelease(button) => {
                        let bit = rdev_button_bit(button);
                        if bit != 0 {
                            let mut btns = ctx.button_state.lock().unwrap();
                            *btns &= !bit;
                            let buttons = *btns;
                            drop(btns);

                            let state = ctx.app_state.lock().unwrap();
                            let lock_mode = state.lock_mode;
                            let mouse_enabled = state.passthrough.enable_mouse;
                            drop(state);

                            if lock_mode && mouse_enabled {
                                let payload = protocol::encode_mouse(buttons, 0, 0, 0);
                                let ble = ctx.ble.clone();
                                rt.spawn(async move {
                                    let ble = ble.lock().await;
                                    let _ = ble.write_mouse(payload).await;
                                });
                                return None;
                            }
                        }
                        return Some(event);
                    }
                    EventType::Wheel { delta_x, delta_y } => {
                        let state = ctx.app_state.lock().unwrap();
                        let lock_mode = state.lock_mode;
                        let mouse_enabled = state.passthrough.enable_mouse;
                        drop(state);

                        if lock_mode && mouse_enabled {
                            let w = (delta_y as i8).clamp(-127, 127);
                            let buttons = *ctx.button_state.lock().unwrap();
                            let payload = protocol::encode_mouse(buttons, 0, 0, w);
                            let ble = ctx.ble.clone();
                            rt.spawn(async move {
                                let ble = ble.lock().await;
                                let _ = ble.write_mouse(payload).await;
                            });
                            // Also suppress horizontal scroll if needed
                            let _ = delta_x;
                            return None;
                        }
                        return Some(event);
                    }
                    // ------ Keyboard events ------
                    EventType::KeyPress(key) => {
                        if is_ctrl(key) { ctx.ctrl_held.store(true, Ordering::SeqCst); }
                        if is_alt(key) { ctx.alt_held.store(true, Ordering::SeqCst); }

                        if ctx.ctrl_held.load(Ordering::SeqCst) && ctx.alt_held.load(Ordering::SeqCst) {
                            let mut state = ctx.app_state.lock().unwrap();
                            state.lock_mode = !state.lock_mode;
                            let new_mode = state.lock_mode;
                            drop(state);
                            let _ = app_handle.emit("lock-mode-changed", new_mode);
                            log::info!("Lock mode toggled to: {}", new_mode);
                            return Some(event);
                        }
                    }
                    EventType::KeyRelease(key) => {
                        if is_ctrl(key) { ctx.ctrl_held.store(false, Ordering::SeqCst); }
                        if is_alt(key) { ctx.alt_held.store(false, Ordering::SeqCst); }
                    }
                }

                let state = ctx.app_state.lock().unwrap();
                let lock_mode = state.lock_mode;
                let kb_enabled = state.passthrough.enable_keyboard;
                drop(state);

                if !lock_mode || !kb_enabled {
                    return Some(event);
                }

                match event.event_type {
                    EventType::KeyPress(key) => {
                        if let Some(mod_bit) = protocol::modifier_bits_from_rdev(key) {
                            let mut mods = ctx.modifier_state.lock().unwrap();
                            *mods |= mod_bit;
                            let payload = protocol::encode_keyboard(*mods, 0);
                            let ble = ctx.ble.clone();
                            rt.spawn(async move {
                                let ble = ble.lock().await;
                                let _ = ble.write_keyboard(payload).await;
                            });
                        } else if let Some(keycode) = protocol::keycode_from_rdev(key) {
                            let mods = *ctx.modifier_state.lock().unwrap();
                            let payload = protocol::encode_keyboard(mods, keycode);
                            let ble = ctx.ble.clone();
                            rt.spawn(async move {
                                let ble = ble.lock().await;
                                let _ = ble.write_keyboard(payload).await;
                            });
                        }
                        None
                    }
                    EventType::KeyRelease(key) => {
                        if let Some(mod_bit) = protocol::modifier_bits_from_rdev(key) {
                            let mut mods = ctx.modifier_state.lock().unwrap();
                            *mods &= !mod_bit;
                            let payload = protocol::encode_keyboard(*mods, 0);
                            let ble = ctx.ble.clone();
                            rt.spawn(async move {
                                let ble = ble.lock().await;
                                let _ = ble.write_keyboard(payload).await;
                            });
                        } else if protocol::keycode_from_rdev(key).is_some() {
                            let mods = *ctx.modifier_state.lock().unwrap();
                            let payload = protocol::encode_keyboard(mods, 0);
                            let ble = ctx.ble.clone();
                            rt.spawn(async move {
                                let ble = ble.lock().await;
                                let _ = ble.write_keyboard(payload).await;
                            });
                        }
                        None
                    }
                    _ => Some(event),
                }
            };

            if let Err(e) = rdev::grab(callback) {
                log::error!("Input grab error: {:?}", e);
                HOOK_RUNNING.store(false, Ordering::SeqCst);
            }
        });
    }

    Ok(())
}


pub fn stop_hook() {
    HOOK_RUNNING.store(false, Ordering::SeqCst);
}

pub fn is_running() -> bool {
    HOOK_RUNNING.load(Ordering::SeqCst)
}
