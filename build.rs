use std::env;
use std::fs;
use std::path::PathBuf;

const DEFAULT_TOML_CONTENT: &str = r#"# Elden Ring Bullet Time Mod Configuration (TOML)

[bullet_time]
# Action mode: "hold" (press and hold to activate) or "toggle" (press once to turn on, press again to turn off)
action_type = "hold"

# Speed multipliers
bullet_time_speed = 0.2
normal_speed = 1.0

# Stealth / Invisibility options during bullet time
# Enable stealth effect during bullet time
enable_stealth = true

# SpEffect IDs to apply for stealth during bullet time (e.g. [4100, 4101])
# 4100: Assassin's Approach / Concealing Veil (reduces enemy detection & eliminates footstep sound)
# 4101: Unseen Form (visual transparency & reduces enemy detection)
stealth_speffect_ids = [4100]

# Key combinations to activate bullet time
# Supports Keyboard ("1", "O", "F1", "Shift+O", etc.) and Xbox Gamepad ("lthumbpress+xa", "PadRSUp", etc.)
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
"#;

fn main() {
    println!("cargo:rerun-if-changed=build.rs");

    // OUT_DIR is typically target/<triple>/<profile>/build/<crate-hash>/out
    let out_dir = match env::var("OUT_DIR") {
        Ok(dir) => dir,
        Err(_) => return,
    };

    let out_path = PathBuf::from(out_dir);

    // Navigate up 3 ancestor levels to get target/<profile> (e.g. target/debug or target/release)
    if let Some(target_dir) = out_path.ancestors().nth(3) {
        let dest_toml_path = target_dir.join("er_bullet_time.toml");
        let _ = fs::write(&dest_toml_path, DEFAULT_TOML_CONTENT);
    }
}
