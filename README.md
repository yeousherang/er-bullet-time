# er-bullet-time

`er-bullet-time` is a Rust-based native DLL modification for Elden Ring that introduces bullet-time (time dilation / slow-motion) mechanics. It selectively slows down world characters and enemies while maintaining normal movement speed for the local player.

The mod is built using the `fromsoftware-rs` bindings framework (`eldenring` and `fromsoftware-shared` crates) and features zero-lag input detection via `windows-sys` for both keyboard and Xbox controller inputs.

---

## Features

- **Bullet-Time Time Dilation**: Alters enemy and world animation speed multipliers dynamically without affecting the player's animation speed.
- **Status Effect Application**: Applies special status effects (`SP_EFFECT`) to the main player character upon activation.
- **Dual Input Support**: Real-time per-frame polling for both Virtual Keys (Keyboard) and XInput (Xbox Controller buttons, analog triggers, and thumbstick directions).
- **Combination Keybinds**: Supports single key triggers as well as multi-button combo strings (e.g., `lthumbpress+xa`).
- **Flexible Action Modes**: Offers both `hold` mode (active while holding key/button) and `toggle` mode (press once to activate, press again to deactivate).
- **Type-Safe Configuration**: Fully configurable via `er_bullet_time.toml` powered by `serde` and `toml`.
- **Automated Build Toolchain**: Cargo build script (`build.rs`) automatically generates and places default configuration files alongside output binaries.

---

## Requirements

- Elden Ring (v1.10+ / current Steam version)
- ModEngine 3 (ME3) or any compatible DLL loader

---

## Installation (ModEngine 3)

1. Download the latest release package (`er-bullet-time-vX.Y.Z.zip`) from the GitHub Releases page.
2. Extract `er_bullet_time.dll` and `er_bullet_time.toml` from the zip file.
3. Place both files into your ModEngine 3 DLL directory (e.g., `modengine3/dlls/` or configured external DLL path).
4. Launch Elden Ring through ModEngine 3.

---

## Configuration (`er_bullet_time.toml`)

The configuration file is automatically created in the same directory as `er_bullet_time.dll` if it does not already exist.

```toml
# Elden Ring Bullet Time Mod Configuration (TOML)

[bullet_time]
# Action mode: "hold" (press and hold to activate) or "toggle" (press once to turn on, press again to turn off)
action_type = "hold"

# Speed multipliers (0.0 = completely frozen, 0.2 = 20% speed, 1.0 = normal speed)
bullet_time_speed = 0.2
normal_speed = 1.0

# Key combinations to activate bullet time
bullet_time_keys = [
    "O",
    "lthumbpress+xa",
    "PadRSUp"
]

# Key combinations to deactivate bullet time
normal_keys = [
    "P",
    "lthumbpress+xb",
    "PadRSDown"
]
```

### Keybind Reference

#### Keyboard
- Single letters: `"O"`, `"P"`, `"A"`, `"Z"`
- Special keys: `"space"`, `"tab"`, `"esc"`
- Hex virtual key codes: `"0x4F"`, `"0x50"`

#### Xbox Controller
- Buttons: `"PadA"`, `"PadB"`, `"PadX"`, `"PadY"`, `"PadLB"`, `"PadRB"`, `"PadStart"`, `"PadBack"`
- Thumbstick Presses (L3 / R3): `"lthumbpress"` (`"lthumb"`), `"rthumbpress"` (`"rthumb"`)
- Analog Triggers: `"PadLT"`, `"PadRT"`
- D-Pad Directions: `"PadDpadUp"`, `"PadDpadDown"`, `"PadDpadLeft"`, `"PadDpadRight"`
- Left Thumbstick Directions: `"PadLSUp"`, `"PadLSDown"`, `"PadLSLeft"`, `"PadLSRight"`
- Right Thumbstick Directions: `"PadRSUp"`, `"PadRSDown"`, `"PadRSLeft"`, `"PadRSRight"`
- Combination Strings: Combine buttons using `+` (e.g., `"lthumbpress+xa"`)

---

## Building from Source

### Prerequisites

- Rust (edition 2024 / stable 1.85+)
- MSVC C++ Build Tools (Windows)

### Build Commands

Clone the repository and build using Cargo:

```bash
git clone https://github.com/your-username/er-bullet-time.git
cd er-bullet-time

# Build Debug version
cargo build

# Build Release version
cargo build --release
```

The output DLL (`er_bullet_time.dll`) and configuration file (`er_bullet_time.toml`) will be located in `target/debug/` or `target/release/`.

---

## License

This project is licensed under the Apache 2.0 License.
