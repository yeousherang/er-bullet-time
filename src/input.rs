use std::collections::HashMap;
use std::sync::Mutex;
use windows_sys::Win32::Foundation::ERROR_SUCCESS;
use windows_sys::Win32::UI::Input::KeyboardAndMouse::GetAsyncKeyState;
use windows_sys::Win32::UI::Input::XboxController::{
    XINPUT_GAMEPAD, XINPUT_GAMEPAD_A, XINPUT_GAMEPAD_B, XINPUT_GAMEPAD_BACK,
    XINPUT_GAMEPAD_DPAD_DOWN, XINPUT_GAMEPAD_DPAD_LEFT, XINPUT_GAMEPAD_DPAD_RIGHT,
    XINPUT_GAMEPAD_DPAD_UP, XINPUT_GAMEPAD_LEFT_SHOULDER, XINPUT_GAMEPAD_LEFT_THUMB,
    XINPUT_GAMEPAD_LEFT_THUMB_DEADZONE, XINPUT_GAMEPAD_RIGHT_SHOULDER, XINPUT_GAMEPAD_RIGHT_THUMB,
    XINPUT_GAMEPAD_RIGHT_THUMB_DEADZONE, XINPUT_GAMEPAD_START, XINPUT_GAMEPAD_TRIGGER_THRESHOLD,
    XINPUT_GAMEPAD_X, XINPUT_GAMEPAD_Y, XINPUT_STATE, XInputGetState,
};

/// Represents key/button states across consecutive frames.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum KeyState {
    /// Key is up and was up in the previous frame.
    Idle,
    /// Key transitioned from up to down in the current frame (Just Pressed).
    Press,
    /// Key remains down from the previous frame (Holding).
    Hold,
    /// Key transitioned from down to up in the current frame (Just Released).
    Release,
}

impl KeyState {
    pub fn is_down(&self) -> bool {
        matches!(self, KeyState::Press | KeyState::Hold)
    }

    pub fn is_pressed(&self) -> bool {
        matches!(self, KeyState::Press)
    }

    pub fn is_released(&self) -> bool {
        matches!(self, KeyState::Release)
    }

    pub fn is_idle(&self) -> bool {
        matches!(self, KeyState::Idle)
    }
}

/// Unified input key representation covering Keyboard VK codes and Xbox Controller inputs.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum InputKey {
    /// Virtual Key Code for Keyboard (e.g. 0x4F for 'O', 0x50 for 'P')
    Keyboard(i32),

    /// Xbox Gamepad Bitmask Buttons
    PadButton(u16),

    /// Xbox Left Trigger (LT) analog threshold
    PadLT,
    /// Xbox Right Trigger (RT) analog threshold
    PadRT,

    /// Xbox Left Thumbstick Directions
    PadLSUp,
    PadLSDown,
    PadLSLeft,
    PadLSRight,

    /// Xbox Right Thumbstick Directions
    PadRSUp,
    PadRSDown,
    PadRSLeft,
    PadRSRight,
}

impl InputKey {
    pub const PAD_DPAD_UP: InputKey = InputKey::PadButton(XINPUT_GAMEPAD_DPAD_UP as u16);
    pub const PAD_DPAD_DOWN: InputKey = InputKey::PadButton(XINPUT_GAMEPAD_DPAD_DOWN as u16);
    pub const PAD_DPAD_LEFT: InputKey = InputKey::PadButton(XINPUT_GAMEPAD_DPAD_LEFT as u16);
    pub const PAD_DPAD_RIGHT: InputKey = InputKey::PadButton(XINPUT_GAMEPAD_DPAD_RIGHT as u16);
    pub const PAD_START: InputKey = InputKey::PadButton(XINPUT_GAMEPAD_START as u16);
    pub const PAD_BACK: InputKey = InputKey::PadButton(XINPUT_GAMEPAD_BACK as u16);
    pub const PAD_LEFT_THUMB: InputKey = InputKey::PadButton(XINPUT_GAMEPAD_LEFT_THUMB as u16);
    pub const PAD_RIGHT_THUMB: InputKey = InputKey::PadButton(XINPUT_GAMEPAD_RIGHT_THUMB as u16);
    pub const PAD_LB: InputKey = InputKey::PadButton(XINPUT_GAMEPAD_LEFT_SHOULDER as u16);
    pub const PAD_RB: InputKey = InputKey::PadButton(XINPUT_GAMEPAD_RIGHT_SHOULDER as u16);
    pub const PAD_A: InputKey = InputKey::PadButton(XINPUT_GAMEPAD_A as u16);
    pub const PAD_B: InputKey = InputKey::PadButton(XINPUT_GAMEPAD_B as u16);
    pub const PAD_X: InputKey = InputKey::PadButton(XINPUT_GAMEPAD_X as u16);
    pub const PAD_Y: InputKey = InputKey::PadButton(XINPUT_GAMEPAD_Y as u16);
}

/// Poll Xbox XInput state for user index (default: 0).
fn get_xinput_state(user_index: u32) -> Option<XINPUT_GAMEPAD> {
    unsafe {
        let mut state: XINPUT_STATE = std::mem::zeroed();
        if XInputGetState(user_index, &mut state) == ERROR_SUCCESS {
            Some(state.Gamepad)
        } else {
            None
        }
    }
}

/// Evaluates if a given input key is currently down (raw state).
fn is_input_raw_down(key: InputKey, gamepad: Option<&XINPUT_GAMEPAD>) -> bool {
    match key {
        InputKey::Keyboard(vk) => unsafe { (GetAsyncKeyState(vk) as u16 & 0x8000) != 0 },
        InputKey::PadButton(mask) => gamepad.map_or(false, |pad| (pad.wButtons & mask) != 0),
        InputKey::PadLT => gamepad.map_or(false, |pad| {
            pad.bLeftTrigger > XINPUT_GAMEPAD_TRIGGER_THRESHOLD as u8
        }),
        InputKey::PadRT => gamepad.map_or(false, |pad| {
            pad.bRightTrigger > XINPUT_GAMEPAD_TRIGGER_THRESHOLD as u8
        }),
        InputKey::PadLSUp => gamepad.map_or(false, |pad| {
            pad.sThumbLY > XINPUT_GAMEPAD_LEFT_THUMB_DEADZONE as i16
        }),
        InputKey::PadLSDown => gamepad.map_or(false, |pad| {
            pad.sThumbLY < -(XINPUT_GAMEPAD_LEFT_THUMB_DEADZONE as i16)
        }),
        InputKey::PadLSLeft => gamepad.map_or(false, |pad| {
            pad.sThumbLX < -(XINPUT_GAMEPAD_LEFT_THUMB_DEADZONE as i16)
        }),
        InputKey::PadLSRight => gamepad.map_or(false, |pad| {
            pad.sThumbLX > XINPUT_GAMEPAD_LEFT_THUMB_DEADZONE as i16
        }),
        InputKey::PadRSUp => gamepad.map_or(false, |pad| {
            pad.sThumbRY > XINPUT_GAMEPAD_RIGHT_THUMB_DEADZONE as i16
        }),
        InputKey::PadRSDown => gamepad.map_or(false, |pad| {
            pad.sThumbRY < -(XINPUT_GAMEPAD_RIGHT_THUMB_DEADZONE as i16)
        }),
        InputKey::PadRSLeft => gamepad.map_or(false, |pad| {
            pad.sThumbRX < -(XINPUT_GAMEPAD_RIGHT_THUMB_DEADZONE as i16)
        }),
        InputKey::PadRSRight => gamepad.map_or(false, |pad| {
            pad.sThumbRX > XINPUT_GAMEPAD_RIGHT_THUMB_DEADZONE as i16
        }),
    }
}

/// Converts string representations to InputKey variants.
pub fn parse_input_key(s: &str) -> Option<InputKey> {
    let s = s.trim().to_lowercase();
    match s.as_str() {
        "a" => Some(InputKey::Keyboard(0x41)),
        "b" => Some(InputKey::Keyboard(0x42)),
        "c" => Some(InputKey::Keyboard(0x43)),
        "d" => Some(InputKey::Keyboard(0x44)),
        "e" => Some(InputKey::Keyboard(0x45)),
        "f" => Some(InputKey::Keyboard(0x46)),
        "g" => Some(InputKey::Keyboard(0x47)),
        "h" => Some(InputKey::Keyboard(0x48)),
        "i" => Some(InputKey::Keyboard(0x49)),
        "j" => Some(InputKey::Keyboard(0x4A)),
        "k" => Some(InputKey::Keyboard(0x4B)),
        "l" => Some(InputKey::Keyboard(0x4C)),
        "m" => Some(InputKey::Keyboard(0x4D)),
        "n" => Some(InputKey::Keyboard(0x4E)),
        "o" => Some(InputKey::Keyboard(0x4F)),
        "p" => Some(InputKey::Keyboard(0x50)),
        "q" => Some(InputKey::Keyboard(0x51)),
        "r" => Some(InputKey::Keyboard(0x52)),
        "s" => Some(InputKey::Keyboard(0x53)),
        "t" => Some(InputKey::Keyboard(0x54)),
        "u" => Some(InputKey::Keyboard(0x55)),
        "v" => Some(InputKey::Keyboard(0x56)),
        "w" => Some(InputKey::Keyboard(0x57)),
        "x" => Some(InputKey::Keyboard(0x58)),
        "y" => Some(InputKey::Keyboard(0x59)),
        "z" => Some(InputKey::Keyboard(0x5A)),
        "space" => Some(InputKey::Keyboard(0x20)),
        "tab" => Some(InputKey::Keyboard(0x09)),
        "escape" | "esc" => Some(InputKey::Keyboard(0x1B)),

        // Xbox Gamepad Aliases (matching er_bullet_time.ini styles like "lthumbpress", "xa", "xb")
        "lthumbpress" | "lthumb" | "padlthumb" | "l3" => Some(InputKey::PAD_LEFT_THUMB),
        "rthumbpress" | "rthumb" | "padrthumb" | "r3" => Some(InputKey::PAD_RIGHT_THUMB),
        "xa" | "pada" => Some(InputKey::PAD_A),
        "xb" | "padb" => Some(InputKey::PAD_B),
        "xx" | "padx" => Some(InputKey::PAD_X),
        "xy" | "pady" => Some(InputKey::PAD_Y),
        "lb" | "padlb" => Some(InputKey::PAD_LB),
        "rb" | "padrb" => Some(InputKey::PAD_RB),
        "lt" | "padlt" => Some(InputKey::PadLT),
        "rt" | "padrt" => Some(InputKey::PadRT),
        "start" | "padstart" => Some(InputKey::PAD_START),
        "back" | "padback" => Some(InputKey::PAD_BACK),
        "dpadup" | "paddpadup" => Some(InputKey::PAD_DPAD_UP),
        "dpaddown" | "paddpaddown" => Some(InputKey::PAD_DPAD_DOWN),
        "dpadleft" | "paddpadleft" => Some(InputKey::PAD_DPAD_LEFT),
        "dpadright" | "paddpadright" => Some(InputKey::PAD_DPAD_RIGHT),
        "lsup" | "padlsup" => Some(InputKey::PadLSUp),
        "lsdown" | "padlsdown" => Some(InputKey::PadLSDown),
        "lsleft" | "padlsleft" => Some(InputKey::PadLSLeft),
        "lsright" | "padlsright" => Some(InputKey::PadLSRight),
        "rsup" | "padrsup" => Some(InputKey::PadRSUp),
        "rsdown" | "padrsdown" => Some(InputKey::PadRSDown),
        "rsleft" | "padrsleft" => Some(InputKey::PadRSLeft),
        "rsright" | "padrsright" => Some(InputKey::PadRSRight),
        other => {
            if let Some(hex_str) = other.strip_prefix("0x") {
                i32::from_str_radix(hex_str, 16)
                    .ok()
                    .map(InputKey::Keyboard)
            } else {
                other.parse::<i32>().ok().map(InputKey::Keyboard)
            }
        }
    }
}

/// Parses key combinations separated by '+' (e.g. "lthumbpress+xa").
pub fn parse_key_combo(s: &str) -> Vec<InputKey> {
    s.split('+').filter_map(parse_input_key).collect()
}

/// Frame-based input manager tracking per-key state transitions.
pub struct InputManager {
    prev_states: HashMap<InputKey, bool>,
    user_index: u32,
}

impl InputManager {
    pub fn new() -> Self {
        Self {
            prev_states: HashMap::new(),
            user_index: 0,
        }
    }

    pub fn set_user_index(&mut self, user_index: u32) {
        self.user_index = user_index;
    }

    pub fn poll_key(&mut self, key: InputKey) -> KeyState {
        let gamepad = get_xinput_state(self.user_index);
        let curr_down = is_input_raw_down(key, gamepad.as_ref());
        let prev_down = self.prev_states.get(&key).copied().unwrap_or(false);

        let state = match (prev_down, curr_down) {
            (false, false) => KeyState::Idle,
            (false, true) => KeyState::Press,
            (true, true) => KeyState::Hold,
            (true, false) => KeyState::Release,
        };

        self.prev_states.insert(key, curr_down);
        state
    }

    /// Evaluates whether a multi-key combination (e.g. "lthumbpress+xa") was just triggered/pressed down.
    pub fn is_combo_pressed(&mut self, combo_str: &str) -> bool {
        let keys = parse_key_combo(combo_str);
        if keys.is_empty() {
            return false;
        }

        let gamepad = get_xinput_state(self.user_index);
        let mut all_curr_down = true;
        let mut any_just_pressed = false;

        for &key in &keys {
            let curr_down = is_input_raw_down(key, gamepad.as_ref());
            let prev_down = self.prev_states.get(&key).copied().unwrap_or(false);

            if !curr_down {
                all_curr_down = false;
            }
            if !prev_down && curr_down {
                any_just_pressed = true;
            }

            self.prev_states.insert(key, curr_down);
        }

        all_curr_down && any_just_pressed
    }

    /// Evaluates whether a multi-key combination is currently held down.
    pub fn is_combo_down(&mut self, combo_str: &str) -> bool {
        let keys = parse_key_combo(combo_str);
        if keys.is_empty() {
            return false;
        }

        let gamepad = get_xinput_state(self.user_index);
        let mut all_curr_down = true;

        for &key in &keys {
            let curr_down = is_input_raw_down(key, gamepad.as_ref());
            if !curr_down {
                all_curr_down = false;
            }
            self.prev_states.insert(key, curr_down);
        }

        all_curr_down
    }
}

static GLOBAL_INPUT_MANAGER: Mutex<Option<InputManager>> = Mutex::new(None);

pub fn poll_key(key: InputKey) -> KeyState {
    let mut guard = GLOBAL_INPUT_MANAGER
        .lock()
        .unwrap_or_else(|e| e.into_inner());
    let manager = guard.get_or_insert_with(InputManager::new);
    manager.poll_key(key)
}

pub fn is_combo_pressed(combo_str: &str) -> bool {
    let mut guard = GLOBAL_INPUT_MANAGER
        .lock()
        .unwrap_or_else(|e| e.into_inner());
    let manager = guard.get_or_insert_with(InputManager::new);
    manager.is_combo_pressed(combo_str)
}

pub fn is_combo_down(combo_str: &str) -> bool {
    let mut guard = GLOBAL_INPUT_MANAGER
        .lock()
        .unwrap_or_else(|e| e.into_inner());
    let manager = guard.get_or_insert_with(InputManager::new);
    manager.is_combo_down(combo_str)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_key_state_helpers() {
        assert!(KeyState::Press.is_down());
        assert!(KeyState::Hold.is_down());
        assert!(!KeyState::Idle.is_down());
        assert!(!KeyState::Release.is_down());

        assert!(KeyState::Press.is_pressed());
        assert!(!KeyState::Hold.is_pressed());

        assert!(KeyState::Release.is_released());
        assert!(KeyState::Idle.is_idle());
    }

    #[test]
    fn test_parse_key_combo() {
        let keys = parse_key_combo("lthumbpress+xa");
        assert_eq!(keys.len(), 2);
        assert_eq!(keys[0], InputKey::PAD_LEFT_THUMB);
        assert_eq!(keys[1], InputKey::PAD_A);
    }
}
