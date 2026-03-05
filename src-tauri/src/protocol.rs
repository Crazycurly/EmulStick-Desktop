/// Encode a keyboard HID packet.
/// Format: [modifiers, 0x00, keycode, 0, 0, 0, 0, 0]
pub fn encode_keyboard(modifiers: u8, keycode: u8) -> [u8; 8] {
    [modifiers, 0x00, keycode, 0, 0, 0, 0, 0]
}

/// Encode a key release (all zeros).
#[allow(dead_code)]
pub fn encode_key_release() -> [u8; 8] {
    [0u8; 8]
}

/// Encode a mouse HID packet.
/// Format: [buttons, x_lo, x_hi, y_lo, y_hi, wheel, 0, 0]
pub fn encode_mouse(buttons: u8, dx: i16, dy: i16, wheel: i8) -> [u8; 8] {
    let x_bytes = dx.to_le_bytes();
    let y_bytes = dy.to_le_bytes();
    [
        buttons,
        x_bytes[0],
        x_bytes[1],
        y_bytes[0],
        y_bytes[1],
        wheel as u8,
        0,
        0,
    ]
}

use rdev::Key;

/// Map rdev Key to USB HID keycode.
pub fn keycode_from_rdev(key: Key) -> Option<u8> {
    match key {
        Key::KeyA => Some(4),
        Key::KeyB => Some(5),
        Key::KeyC => Some(6),
        Key::KeyD => Some(7),
        Key::KeyE => Some(8),
        Key::KeyF => Some(9),
        Key::KeyG => Some(10),
        Key::KeyH => Some(11),
        Key::KeyI => Some(12),
        Key::KeyJ => Some(13),
        Key::KeyK => Some(14),
        Key::KeyL => Some(15),
        Key::KeyM => Some(16),
        Key::KeyN => Some(17),
        Key::KeyO => Some(18),
        Key::KeyP => Some(19),
        Key::KeyQ => Some(20),
        Key::KeyR => Some(21),
        Key::KeyS => Some(22),
        Key::KeyT => Some(23),
        Key::KeyU => Some(24),
        Key::KeyV => Some(25),
        Key::KeyW => Some(26),
        Key::KeyX => Some(27),
        Key::KeyY => Some(28),
        Key::KeyZ => Some(29),
        Key::Num1 => Some(30),
        Key::Num2 => Some(31),
        Key::Num3 => Some(32),
        Key::Num4 => Some(33),
        Key::Num5 => Some(34),
        Key::Num6 => Some(35),
        Key::Num7 => Some(36),
        Key::Num8 => Some(37),
        Key::Num9 => Some(38),
        Key::Num0 => Some(39),
        Key::Return => Some(40),
        Key::Escape => Some(41),
        Key::Backspace => Some(42),
        Key::Tab => Some(43),
        Key::Space => Some(44),
        Key::Minus => Some(45),
        Key::Equal => Some(46),
        Key::LeftBracket => Some(47),
        Key::RightBracket => Some(48),
        Key::BackSlash => Some(49),
        Key::SemiColon => Some(51),
        Key::Quote => Some(52),
        Key::BackQuote => Some(53),
        Key::Comma => Some(54),
        Key::Dot => Some(55),
        Key::Slash => Some(56),
        Key::CapsLock => Some(57),
        Key::F1 => Some(58),
        Key::F2 => Some(59),
        Key::F3 => Some(60),
        Key::F4 => Some(61),
        Key::F5 => Some(62),
        Key::F6 => Some(63),
        Key::F7 => Some(64),
        Key::F8 => Some(65),
        Key::F9 => Some(66),
        Key::F10 => Some(67),
        Key::F11 => Some(68),
        Key::F12 => Some(69),
        Key::PrintScreen => Some(70),
        Key::ScrollLock => Some(71),
        Key::Pause => Some(72),
        Key::Insert => Some(73),
        Key::Home => Some(74),
        Key::PageUp => Some(75),
        Key::Delete => Some(76),
        Key::End => Some(77),
        Key::PageDown => Some(78),
        Key::RightArrow => Some(79),
        Key::LeftArrow => Some(80),
        Key::DownArrow => Some(81),
        Key::UpArrow => Some(82),
        _ => None,
    }
}

/// Check if a key is a modifier and return its bit position.
pub fn modifier_bits_from_rdev(key: Key) -> Option<u8> {
    match key {
        Key::ControlLeft => Some(0x01),  // bit 0
        Key::ShiftLeft => Some(0x02),    // bit 1
        Key::Alt => Some(0x04),          // bit 2
        Key::MetaLeft => Some(0x08),     // bit 3 (Left GUI / Win / Cmd)
        Key::ControlRight => Some(0x10), // bit 4
        Key::ShiftRight => Some(0x20),   // bit 5
        Key::AltGr => Some(0x40),        // bit 6
        Key::MetaRight => Some(0x80),    // bit 7
        _ => None,
    }
}

/// Map rdev mouse button to HID button bitmask.
#[allow(dead_code)]
pub fn mouse_button_bit(button: rdev::Button) -> u8 {
    match button {
        rdev::Button::Left => 0x01,
        rdev::Button::Right => 0x02,
        rdev::Button::Middle => 0x04,
        _ => 0x00,
    }
}
