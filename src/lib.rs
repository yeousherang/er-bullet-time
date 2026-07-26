use eldenring::{
    cs::{CSTaskGroupIndex, CSTaskImp, ChrInsExt, ChrType, WorldChrMan},
    fd4::FD4TaskData,
};
use fromsoftware_shared::{FromStatic, SharedTaskImpExt};
use std::{
    panic::AssertUnwindSafe,
    sync::OnceLock,
    sync::atomic::{AtomicBool, Ordering},
    time::Duration,
};
use tracing_appender::non_blocking::WorkerGuard;
use tracing_subscriber::{EnvFilter, fmt, layer::SubscriberExt, util::SubscriberInitExt};

pub mod config;
pub mod input;

use config::{ActionMode, get_config};
use input::{is_combo_down, is_combo_pressed};

const SP_EFFECT: i32 = 4330;
static IS_BULLET_TIME_ACTIVE: AtomicBool = AtomicBool::new(false);
static LOG_GUARD: OnceLock<WorkerGuard> = OnceLock::new();

fn init_tracing() {
    let log_dir = config::get_dll_directory().unwrap_or_else(|| std::path::PathBuf::from("logs"));
    let file_appender = tracing_appender::rolling::never(log_dir, "er-bullet-time.log");
    let (file_writer, guard) = tracing_appender::non_blocking(file_appender);

    let console_layer = fmt::layer()
        .with_writer(std::io::stdout)
        .with_target(true)
        .with_thread_ids(true)
        .with_thread_names(true);

    let file_layer = fmt::layer()
        .with_writer(file_writer)
        .with_ansi(false)
        .with_target(true)
        .with_thread_ids(true)
        .with_thread_names(true);

    let filter = EnvFilter::try_from_default_env().unwrap_or_else(|_| EnvFilter::new("info"));

    tracing_subscriber::registry()
        .with(filter)
        .with(console_layer)
        .with(file_layer)
        .init();

    let _ = LOG_GUARD.set(guard);
}

fn set_chr_animation_speeds(world_chr_man: &mut WorldChrMan, normal_speed: f32, target_speed: f32) {
    let cfg = &get_config().bullet_time;
    let include_torrent = cfg.include_torrent;

    let mount_handle = world_chr_man
        .main_player
        .as_ref()
        .map(|p| unsafe { p.player_game_data.as_ref().mount_handle });

    for chr_set in world_chr_man.chr_sets.iter().flatten() {
        for chr in chr_set.characters() {
            if chr as *const _ as usize == 0 {
                continue;
            }

            let _ = std::panic::catch_unwind(AssertUnwindSafe(move || {
                let is_local_player = chr.chr_type == ChrType::Local;
                let is_torrent = include_torrent
                    && (chr.npc_id == 8000
                        || chr.character_id == 8000
                        || mount_handle
                            .map_or(false, |h| !h.is_empty() && h == chr.field_ins_handle));

                chr.modules.behavior.animation_speed = if is_local_player || is_torrent {
                    normal_speed
                } else {
                    target_speed
                };
            }));
        }
    }
}

fn enable_bullet_time(world_chr_man: &mut WorldChrMan, normal_speed: f32, target_speed: f32) {
    if IS_BULLET_TIME_ACTIVE.swap(true, Ordering::SeqCst) {
        // Already active, just maintain speeds
        set_chr_animation_speeds(world_chr_man, normal_speed, target_speed);
        return;
    }

    let Some(main_player) = world_chr_man.main_player.as_mut() else {
        return;
    };

    tracing::info!("Bullet time enabled (speed: {})", target_speed);
    main_player.apply_speffect(SP_EFFECT, true);

    let cfg = &get_config().bullet_time;
    if cfg.enable_stealth {
        for &sp_id in &cfg.stealth_speffect_ids {
            main_player.apply_speffect(sp_id, false);
        }
    }

    set_chr_animation_speeds(world_chr_man, normal_speed, target_speed);
}

fn disable_bullet_time(world_chr_man: &mut WorldChrMan, normal_speed: f32) {
    if !IS_BULLET_TIME_ACTIVE.swap(false, Ordering::SeqCst) {
        // Already inactive, just maintain speeds
        set_chr_animation_speeds(world_chr_man, normal_speed, normal_speed);
        return;
    }

    let Some(main_player) = world_chr_man.main_player.as_mut() else {
        return;
    };

    tracing::info!("Bullet time disabled");
    main_player.chr_ins.remove_speffect(SP_EFFECT);

    let cfg = &get_config().bullet_time;
    if cfg.enable_stealth {
        for &sp_id in &cfg.stealth_speffect_ids {
            main_player.chr_ins.remove_speffect(sp_id);
        }
    }

    set_chr_animation_speeds(world_chr_man, normal_speed, normal_speed);
}

fn update_bullet_time() {
    let Ok(world_chr_man) = (unsafe { WorldChrMan::instance_mut() }) else {
        return;
    };

    let cfg = &get_config().bullet_time;
    let normal_speed = cfg.normal_speed;
    let bullet_speed = cfg.bullet_time_speed;

    let activate_triggered = cfg.bullet_time_keys.iter().any(|k| is_combo_pressed(k));
    let deactivate_triggered = cfg.normal_keys.iter().any(|k| is_combo_pressed(k));

    match cfg.action_type {
        ActionMode::Toggle => {
            if activate_triggered {
                let currently_active = IS_BULLET_TIME_ACTIVE.load(Ordering::SeqCst);
                if currently_active {
                    disable_bullet_time(world_chr_man, normal_speed);
                } else {
                    enable_bullet_time(world_chr_man, normal_speed, bullet_speed);
                }
            } else if deactivate_triggered {
                disable_bullet_time(world_chr_man, normal_speed);
            } else if IS_BULLET_TIME_ACTIVE.load(Ordering::SeqCst) {
                // Maintain bullet time animation speeds
                set_chr_animation_speeds(world_chr_man, normal_speed, bullet_speed);
            }
        }
        ActionMode::Hold => {
            let is_holding_activate = cfg.bullet_time_keys.iter().any(|k| is_combo_down(k));
            if is_holding_activate {
                enable_bullet_time(world_chr_man, normal_speed, bullet_speed);
            } else {
                disable_bullet_time(world_chr_man, normal_speed);
            }
        }
    }
}

/// # Safety
/// Exposed for libraryloader to call. Do not call directly.
#[allow(non_snake_case)]
#[unsafe(no_mangle)]
pub unsafe extern "C" fn DllMain(_hmodule: u64, reason: u32) -> bool {
    if reason != 1 {
        return true;
    }

    init_tracing();
    let _ = get_config(); // Initialize config on DLL load

    std::thread::spawn(move || {
        let cs_task = CSTaskImp::wait_for_instance(Duration::MAX).unwrap();

        cs_task.run_recurring(
            |_: &FD4TaskData| update_bullet_time(),
            CSTaskGroupIndex::FrameBegin,
        );
    });

    true
}
