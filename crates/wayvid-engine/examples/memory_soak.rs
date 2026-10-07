//! Controlled Linux memory probe; run under a disposable X server.
use std::{
    path::PathBuf,
    time::{Duration, Instant},
};
use wayvid_engine::{spawn_engine, EngineCommand, EngineConfig, EngineEvent, HwdecMode};

fn main() -> anyhow::Result<()> {
    let args: Vec<_> = std::env::args().collect();
    let path = PathBuf::from(args.get(1).expect("video path"));
    let cycling = args.get(2).is_some_and(|s| s == "cycle");
    let seconds: u64 = args.get(3).map_or(Ok(780), |s| s.parse())?;
    let interval: u64 = args.get(4).map_or(Ok(3), |s| s.parse())?;
    anyhow::ensure!(interval > 0, "cycle interval must be positive");
    let mut config = EngineConfig::default();
    config.video.hwdec = HwdecMode::No;
    config.fps_limit = Some(30);
    config.pause_on_battery = false;
    let (handle, events) = spawn_engine(config)?;
    loop {
        match events.recv_timeout(Duration::from_secs(5))? {
            EngineEvent::Started => break,
            EngineEvent::Error(error) => anyhow::bail!(error),
            _ => {}
        }
    }
    handle.send(EngineCommand::ApplyWallpaper {
        path: path.clone(),
        output: None,
    })?;
    let start = Instant::now();
    for tick in 0..=seconds / 5 {
        let target = Duration::from_secs(tick * 5);
        if target > start.elapsed() {
            std::thread::sleep(target - start.elapsed());
        }
        for event in events.try_iter() {
            if let EngineEvent::Error(error) = event {
                anyhow::bail!(error);
            }
        }
        let memory = std::fs::read_to_string("/proc/self/smaps_rollup")?;
        let stats: Vec<_> = memory
            .lines()
            .filter(|s| {
                ["Rss:", "Pss:", "Anonymous:", "Swap:"]
                    .iter()
                    .any(|key| s.starts_with(key))
            })
            .collect();
        println!(
            "{}\t{}\t{}",
            start.elapsed().as_secs_f64(),
            stats.join("\t"),
            allocator_stats()
        );
        if cycling && tick > 0 && tick % interval == 0 {
            handle.send(EngineCommand::ClearWallpaper { output: None })?;
            std::thread::sleep(Duration::from_millis(250));
            handle.send(EngineCommand::ApplyWallpaper {
                path: path.clone(),
                output: None,
            })?;
        }
    }
    handle.request_shutdown();
    handle.join()?;
    println!("post_shutdown\t{}", allocator_stats());
    // Diagnostic only: demonstrate freed allocator pages after the workload.
    // Production playback never forces allocator trimming.
    #[cfg(target_env = "gnu")]
    if args.get(5).is_some_and(|s| s == "trim") {
        unsafe {
            libc::malloc_trim(0);
        }
        println!(
            "post_trim\t{}\t{}",
            allocator_stats(),
            std::fs::read_to_string("/proc/self/smaps_rollup")?.replace('\n', "\t")
        );
    }
    Ok(())
}

fn allocator_stats() -> String {
    #[cfg(target_env = "gnu")]
    {
        let stats = unsafe { libc::mallinfo2() };
        format!(
            "malloc_live={} malloc_free={} mmap={}",
            stats.uordblks, stats.fordblks, stats.hblkhd
        )
    }
    #[cfg(not(target_env = "gnu"))]
    "malloc_stats_unavailable".into()
}
