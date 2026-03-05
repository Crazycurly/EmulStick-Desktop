use crate::ble::BleState;
use crate::protocol;
use crate::state::SharedAppState;
use rdev::{Event, EventType};
use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::Arc;
use tokio::sync::Mutex;

static HOOK_RUNNING: AtomicBool = AtomicBool::new(false);

struct HookContext {
    ble: Arc<Mutex<BleState>>,
    app_state: SharedAppState,
    modifier_state: std::sync::Mutex<u8>,
    button_state: std::sync::Mutex<u8>,
    last_mouse_pos: std::sync::Mutex<(f64, f64)>,
    has_last_pos: AtomicBool,
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
        last_mouse_pos: std::sync::Mutex::new((0.0, 0.0)),
        has_last_pos: AtomicBool::new(false),
    });

    let rt = tokio::runtime::Handle::current();

    std::thread::spawn(move || {
        let callback = move |event: Event| {
            if !HOOK_RUNNING.load(Ordering::SeqCst) {
                return;
            }

            let ctx = ctx.clone();
            let _app_handle = app_handle.clone();
            let rt = rt.clone();

            match event.event_type {
                EventType::KeyPress(key) => {
                    let state = ctx.app_state.lock().unwrap();
                    if !state.lock_mode || !state.passthrough.enable_keyboard {
                        return;
                    }
                    drop(state);

                    // Check for lock mode toggle: Ctrl+Alt
                    // (handled separately in key release)

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
                }
                EventType::KeyRelease(key) => {
                    let state = ctx.app_state.lock().unwrap();
                    if !state.lock_mode || !state.passthrough.enable_keyboard {
                        return;
                    }
                    drop(state);

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
                        // Send key release (keep current modifiers, zero keycode)
                        let mods = *ctx.modifier_state.lock().unwrap();
                        let payload = protocol::encode_keyboard(mods, 0);
                        let ble = ctx.ble.clone();
                        rt.spawn(async move {
                            let ble = ble.lock().await;
                            let _ = ble.write_keyboard(payload).await;
                        });
                    }
                }
                EventType::MouseMove { x, y } => {
                    let state = ctx.app_state.lock().unwrap();
                    if !state.lock_mode || !state.passthrough.enable_mouse {
                        return;
                    }
                    drop(state);

                    if !ctx.has_last_pos.load(Ordering::SeqCst) {
                        let mut pos = ctx.last_mouse_pos.lock().unwrap();
                        *pos = (x, y);
                        ctx.has_last_pos.store(true, Ordering::SeqCst);
                        return;
                    }

                    let mut last_pos = ctx.last_mouse_pos.lock().unwrap();
                    let dx = (x - last_pos.0) as i16;
                    let dy = (y - last_pos.1) as i16;
                    *last_pos = (x, y);

                    // Clamp to ±2047
                    let dx = dx.clamp(-2047, 2047);
                    let dy = dy.clamp(-2047, 2047);

                    if dx == 0 && dy == 0 {
                        return;
                    }

                    let buttons = *ctx.button_state.lock().unwrap();
                    let payload = protocol::encode_mouse(buttons, dx, dy, 0);
                    let ble = ctx.ble.clone();
                    rt.spawn(async move {
                        let ble = ble.lock().await;
                        let _ = ble.write_mouse(payload).await;
                    });
                }
                EventType::ButtonPress(button) => {
                    let state = ctx.app_state.lock().unwrap();
                    if !state.lock_mode || !state.passthrough.enable_mouse {
                        return;
                    }
                    drop(state);

                    let bit = protocol::mouse_button_bit(button);
                    let mut buttons = ctx.button_state.lock().unwrap();
                    *buttons |= bit;
                    let payload = protocol::encode_mouse(*buttons, 0, 0, 0);
                    let ble = ctx.ble.clone();
                    rt.spawn(async move {
                        let ble = ble.lock().await;
                        let _ = ble.write_mouse(payload).await;
                    });
                }
                EventType::ButtonRelease(button) => {
                    let state = ctx.app_state.lock().unwrap();
                    if !state.lock_mode || !state.passthrough.enable_mouse {
                        return;
                    }
                    drop(state);

                    let bit = protocol::mouse_button_bit(button);
                    let mut buttons = ctx.button_state.lock().unwrap();
                    *buttons &= !bit;
                    let payload = protocol::encode_mouse(*buttons, 0, 0, 0);
                    let ble = ctx.ble.clone();
                    rt.spawn(async move {
                        let ble = ble.lock().await;
                        let _ = ble.write_mouse(payload).await;
                    });
                }
                EventType::Wheel { delta_x: _, delta_y } => {
                    let state = ctx.app_state.lock().unwrap();
                    if !state.lock_mode || !state.passthrough.enable_mouse {
                        return;
                    }
                    drop(state);

                    let wheel = (delta_y as i8).clamp(-127, 127);
                    let payload = protocol::encode_mouse(0, 0, 0, wheel);
                    let ble = ctx.ble.clone();
                    rt.spawn(async move {
                        let ble = ble.lock().await;
                        let _ = ble.write_mouse(payload).await;
                    });
                }
            }
        };

        if let Err(e) = rdev::listen(callback) {
            log::error!("Input hook error: {:?}", e);
            HOOK_RUNNING.store(false, Ordering::SeqCst);
        }
    });

    Ok(())
}

pub fn stop_hook() {
    HOOK_RUNNING.store(false, Ordering::SeqCst);
}

pub fn is_running() -> bool {
    HOOK_RUNNING.load(Ordering::SeqCst)
}
