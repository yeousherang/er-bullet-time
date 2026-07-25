use std::{sync::OnceLock, time::Duration};

use eldenring::{
    cs::{CSTaskGroupIndex, CSTaskImp, ChrInsExt, WorldChrMan},
    fd4::FD4TaskData,
    util::input,
};
use fromsoftware_shared::{FromStatic, SharedTaskImpExt};
use tracing_appender::non_blocking::WorkerGuard;
use tracing_subscriber::{fmt, layer::SubscriberExt, util::SubscriberInitExt, EnvFilter};

const SP_EFFECT: i32 = 4330;

static LOG_GUARD: OnceLock<WorkerGuard> = OnceLock::new();

fn init_tracing() {
    let file_appender = tracing_appender::rolling::never(".", "er-bullet-time.log");
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

    let fitter = EnvFilter::try_from_default_env()
        .unwrap_or_else(|_| EnvFilter::new("info"));

    tracing_subscriber::registry()
        .with(fitter)
        .with(console_layer)
        .with(file_layer)
        .init();

    let _ = LOG_GUARD.set(guard);
}

/// # Safety
/// This is exposed this way such that libraryloader can call it. Do not call this yourself.
#[unsafe(no_mangle)]
pub unsafe extern "C" fn DllMain(_hmodule: u64, reason: u32) -> bool {
    // Exit early if we're not attaching a DLL
    if reason != 1 {
        return true;
    }

    init_tracing();

    std::thread::spawn(move || {
        // Retrieve games task runner and register a task at frame begin.
        let cs_task = CSTaskImp::wait_for_instance(Duration::MAX).unwrap();
        cs_task.run_recurring(
            |_: &FD4TaskData| {
                // Retrieve WorldChrMan
                let Ok(world_chr_man) = (unsafe { WorldChrMan::instance_mut() }) else {
                    return;
                };

                // Retrieve main player
                let Some(ref mut main_player) = world_chr_man.main_player else {
                    return;
                };

                // Check if "o" is pressed
                if input::is_key_pressed(0x4F) {
                    // log
                    tracing::info!("o pressed");
                    main_player.apply_speffect(SP_EFFECT, true);
                }

                // Check if "p" is pressed
                if input::is_key_pressed(0x50) {
                    tracing::info!("p pressed");
                    main_player.chr_ins.remove_speffect(SP_EFFECT);
                }
            },
            CSTaskGroupIndex::FrameBegin,
        );
    });

    true
}
