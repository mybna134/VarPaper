//! One process-wide playback monitor; no microphone or wallpaper-volume input.
mod pulse;

use std::sync::atomic::{AtomicBool, AtomicUsize, Ordering};
use std::sync::{Arc, Mutex, OnceLock, Weak};
use std::thread::JoinHandle;
use std::time::{Duration, Instant};

pub const SAMPLE_RATE: usize = 48_000;
pub const BANDS: usize = 64;
const WINDOW: usize = 2048;
const STALE: Duration = Duration::from_millis(250);
static WORKERS: AtomicUsize = AtomicUsize::new(0);
static CONSUMERS: AtomicUsize = AtomicUsize::new(0);
static SERVICE: OnceLock<Mutex<Weak<Service>>> = OnceLock::new();

#[derive(Clone, Debug)]
pub struct AudioSnapshot {
    pub sequence: u64,
    /// 64 increasing-frequency left bands, followed by 64 right bands; 0..1.
    pub spectrum: [f32; BANDS * 2],
    pub level: [f32; 2],
    pub monitor: Option<String>,
    pub diagnostic: Option<String>,
}
impl Default for AudioSnapshot {
    fn default() -> Self {
        Self {
            sequence: 0,
            spectrum: [0.0; BANDS * 2],
            level: [0.0; 2],
            monitor: None,
            diagnostic: Some("Playback monitor is connecting".into()),
        }
    }
}
struct State {
    stop: AtomicBool,
    snapshot: Mutex<(AudioSnapshot, Instant)>,
}
struct Service {
    state: Arc<State>,
    worker: Option<JoinHandle<()>>,
}
impl Drop for Service {
    fn drop(&mut self) {
        self.state.stop.store(true, Ordering::Release);
        if let Some(worker) = self.worker.take() {
            worker.thread().unpark();
            let _ = worker.join();
        }
    }
}
/// Keeping this handle alive keeps the shared monitor alive. Dropping the last
/// handle synchronously disconnects the stream, context and worker.
pub struct AudioConsumer(Arc<Service>);
impl AudioConsumer {
    pub fn acquire() -> Self {
        let mut registry = SERVICE
            .get_or_init(|| Mutex::new(Weak::new()))
            .lock()
            .unwrap();
        let service = registry.upgrade().unwrap_or_else(|| {
            let state = Arc::new(State {
                stop: AtomicBool::new(false),
                snapshot: Mutex::new((AudioSnapshot::default(), Instant::now())),
            });
            let worker_state = state.clone();
            let worker = std::thread::Builder::new()
                .name("wallpaper-playback-monitor".into())
                .spawn(move || run(worker_state, pulse::Monitor::new));
            let worker = match worker {
                Ok(worker) => Some(worker),
                Err(error) => {
                    state.snapshot.lock().unwrap().0.diagnostic = Some(error.to_string());
                    None
                }
            };
            let service = Arc::new(Service { state, worker });
            *registry = Arc::downgrade(&service);
            service
        });
        CONSUMERS.fetch_add(1, Ordering::Relaxed);
        Self(service)
    }
    pub fn snapshot(&self) -> AudioSnapshot {
        let (mut snapshot, updated) = self.0.state.snapshot.lock().unwrap().clone();
        if updated.elapsed() > STALE {
            snapshot.spectrum.fill(0.0);
            snapshot.level.fill(0.0);
        }
        snapshot
    }
}
impl Drop for AudioConsumer {
    fn drop(&mut self) {
        CONSUMERS.fetch_sub(1, Ordering::Relaxed);
    }
}
pub fn live_counts() -> (usize, usize) {
    (
        CONSUMERS.load(Ordering::Relaxed),
        WORKERS.load(Ordering::Relaxed),
    )
}
enum Input {
    Monitor(String),
    Samples(Vec<f32>),
}
trait Capture {
    fn poll(&mut self) -> anyhow::Result<Option<Input>>;
}
fn run<C: Capture>(state: Arc<State>, mut connect: impl FnMut() -> anyhow::Result<C>) {
    struct Worker;
    impl Drop for Worker {
        fn drop(&mut self) {
            WORKERS.fetch_sub(1, Ordering::Relaxed);
        }
    }
    WORKERS.fetch_add(1, Ordering::Relaxed);
    let _worker = Worker;
    while !state.stop.load(Ordering::Acquire) {
        let mut analyzer = Analyzer::default();
        let result = (|| -> anyhow::Result<()> {
            let mut capture = connect()?;
            while !state.stop.load(Ordering::Acquire) {
                match capture.poll()? {
                    Some(Input::Monitor(name)) => {
                        analyzer = Analyzer::default();
                        *state.snapshot.lock().unwrap() = (
                            AudioSnapshot {
                                monitor: Some(name),
                                diagnostic: None,
                                ..AudioSnapshot::default()
                            },
                            Instant::now(),
                        );
                    }
                    Some(Input::Samples(samples)) => {
                        if let Some((spectrum, level)) = analyzer.feed(&samples) {
                            let mut current = state.snapshot.lock().unwrap();
                            current.0.spectrum = spectrum;
                            current.0.level = level;
                            current.0.sequence = current.0.sequence.saturating_add(1);
                            current.1 = Instant::now();
                        }
                    }
                    None => std::thread::park_timeout(Duration::from_millis(5)),
                }
            }
            Ok(())
        })();
        if let Err(error) = result {
            *state.snapshot.lock().unwrap() = (
                AudioSnapshot {
                    diagnostic: Some(format!("Playback monitor unavailable: {error:#}")),
                    ..AudioSnapshot::default()
                },
                Instant::now(),
            );
            if !state.stop.load(Ordering::Acquire) {
                std::thread::park_timeout(Duration::from_millis(500));
            }
        }
    }
}

#[derive(Default)]
struct Analyzer {
    pending: Vec<f32>,
}
impl Analyzer {
    fn feed(&mut self, samples: &[f32]) -> Option<([f32; BANDS * 2], [f32; 2])> {
        // Capture chunks are stereo-aligned and bounded before reaching here.
        let mut result = None;
        for pair in samples.as_chunks::<2>().0 {
            self.pending.extend(pair.iter().map(|v| {
                if v.is_finite() {
                    v.clamp(-1.0, 1.0)
                } else {
                    0.0
                }
            }));
            if self.pending.len() == WINDOW * 2 {
                result = Some(analyze(&self.pending));
                self.pending.clear();
            }
        }
        result
    }
}
fn analyze(samples: &[f32]) -> ([f32; BANDS * 2], [f32; 2]) {
    let mut spectrum = [0.0; BANDS * 2];
    let mut level = [0.0; 2];
    for channel in 0..2 {
        let mut fft = [(0.0_f32, 0.0_f32); WINDOW];
        let mut energy = 0.0;
        let mut weight = 0.0;
        for (i, bin) in fft.iter_mut().enumerate() {
            let value = samples[i * 2 + channel];
            energy += value * value;
            let hann = 0.5 - 0.5 * (std::f32::consts::TAU * i as f32 / WINDOW as f32).cos();
            weight += hann;
            bin.0 = value * hann;
        }
        level[channel] = (energy / WINDOW as f32).sqrt().clamp(0.0, 1.0);
        // In-place radix-2 FFT with fixed, bounded scratch storage.
        for i in 0..WINDOW {
            let j = i.reverse_bits() >> (usize::BITS - WINDOW.trailing_zeros());
            if j > i {
                fft.swap(i, j);
            }
        }
        let mut size = 2;
        while size <= WINDOW {
            for start in (0..WINDOW).step_by(size) {
                for i in 0..size / 2 {
                    let angle = -std::f32::consts::TAU * i as f32 / size as f32;
                    let (s, c) = angle.sin_cos();
                    let (re, im) = fft[start + i + size / 2];
                    let b = (re * c - im * s, re * s + im * c);
                    let a = fft[start + i];
                    fft[start + i] = (a.0 + b.0, a.1 + b.1);
                    fft[start + i + size / 2] = (a.0 - b.0, a.1 - b.1);
                }
            }
            size *= 2;
        }
        for band in 0..BANDS {
            let start = (band_edge(band) * WINDOW as f32 / SAMPLE_RATE as f32).ceil() as usize;
            let end = (band_edge(band + 1) * WINDOW as f32 / SAMPLE_RATE as f32).ceil() as usize;
            // Low log bands can be narrower than FFT resolution; share nearest bin.
            let start = start.clamp(1, WINDOW / 2);
            let end = end.max(start + 1).min(WINDOW / 2 + 1);
            spectrum[channel * BANDS + band] = fft[start..end]
                .iter()
                .map(|(re, im)| 2.0 * re.hypot(*im) / weight)
                .fold(0.0, f32::max)
                .clamp(0.0, 1.0);
        }
    }
    (spectrum, level)
}
fn band_edge(band: usize) -> f32 {
    20.0 * 1000.0_f32.powf(band as f32 / BANDS as f32)
}

#[cfg(test)]
mod tests {
    use super::*;
    fn tones(left: f32, right: f32) -> Vec<f32> {
        (0..WINDOW)
            .flat_map(|n| {
                [left, right].map(|f| {
                    0.5 * (std::f32::consts::TAU * f * n as f32 / SAMPLE_RATE as f32).sin()
                })
            })
            .collect()
    }
    #[test]
    fn known_stereo_tones_preserve_frequency_channel_order_and_level() {
        let (bands, levels) = Analyzer::default().feed(&tones(375.0, 6000.0)).unwrap();
        for (channel, frequency) in [(0, 375.0), (1, 6000.0)] {
            let peak = (0..BANDS)
                .max_by(|a, b| bands[channel * BANDS + *a].total_cmp(&bands[channel * BANDS + *b]))
                .unwrap();
            assert!(band_edge(peak) <= frequency && band_edge(peak + 1) > frequency);
            assert!((bands[channel * BANDS + peak] - 0.5).abs() < 0.01);
            assert!((levels[channel] - 0.5 / 2.0_f32.sqrt()).abs() < 0.01);
        }
        assert!(bands
            .iter()
            .all(|v| v.is_finite() && (0.0..=1.0).contains(v)));
    }
    #[test]
    fn silence_invalid_samples_and_chunk_boundaries_are_bounded() {
        let mut analyzer = Analyzer::default();
        assert!(analyzer.feed(&vec![f32::NAN; 1024]).is_none());
        let (bands, levels) = analyzer
            .feed(&vec![f32::INFINITY; WINDOW * 2 - 1024])
            .unwrap();
        assert_eq!(bands, [0.0; BANDS * 2]);
        assert_eq!(levels, [0.0; 2]);
        assert!(analyzer.pending.is_empty());
    }
    #[test]
    fn simulated_disconnect_output_switch_and_recovery_reset_inputs() {
        use std::sync::mpsc;
        struct Fake(mpsc::Receiver<anyhow::Result<Input>>);
        impl Capture for Fake {
            fn poll(&mut self) -> anyhow::Result<Option<Input>> {
                match self.0.try_recv() {
                    Ok(value) => value.map(Some),
                    Err(_) => Ok(None),
                }
            }
        }
        let state = Arc::new(State {
            stop: AtomicBool::new(false),
            snapshot: Mutex::new((AudioSnapshot::default(), Instant::now())),
        });
        let (sender, receiver) = mpsc::channel();
        let (retry_sender, retry_receiver) = mpsc::channel();
        let worker_state = state.clone();
        let mut receivers = [receiver, retry_receiver].into_iter();
        let thread =
            std::thread::spawn(move || run(worker_state, || Ok(Fake(receivers.next().unwrap()))));
        fn wait(state: &State, ready: impl Fn(&AudioSnapshot) -> bool) {
            let deadline = Instant::now() + Duration::from_secs(3);
            while !ready(&state.snapshot.lock().unwrap().0) {
                assert!(Instant::now() < deadline, "Audio event timed out");
                std::thread::sleep(Duration::from_millis(5));
            }
        }
        sender
            .send(Ok(Input::Monitor("first.monitor".into())))
            .unwrap();
        sender
            .send(Ok(Input::Samples(tones(375.0, 6000.0))))
            .unwrap();
        wait(&state, |s| s.level[0] > 0.1);
        sender
            .send(Ok(Input::Monitor("second.monitor".into())))
            .unwrap();
        wait(&state, |s| {
            s.monitor.as_deref() == Some("second.monitor") && s.level == [0.0; 2]
        });
        sender.send(Err(anyhow::anyhow!("disconnected"))).unwrap();
        wait(&state, |s| {
            s.diagnostic
                .as_ref()
                .is_some_and(|d| d.contains("disconnected"))
                && s.spectrum == [0.0; BANDS * 2]
        });
        retry_sender
            .send(Ok(Input::Monitor("recovered.monitor".into())))
            .unwrap();
        retry_sender
            .send(Ok(Input::Samples(tones(6000.0, 375.0))))
            .unwrap();
        wait(&state, |s| {
            s.monitor.as_deref() == Some("recovered.monitor")
                && s.level[0] > 0.1
                && s.diagnostic.is_none()
        });
        state.stop.store(true, Ordering::Release);
        thread.thread().unpark();
        thread.join().unwrap();
    }
    #[test]
    fn consumers_share_one_service_and_last_drop_joins_worker() {
        let first = AudioConsumer::acquire();
        let second = AudioConsumer::acquire();
        assert!(Arc::ptr_eq(&first.0, &second.0));
        let weak = Arc::downgrade(&first.0);
        drop(first);
        assert!(weak.upgrade().is_some());
        drop(second);
        assert!(weak.upgrade().is_none());
    }
    #[test]
    fn stale_capture_supplies_zero_without_losing_diagnostic() {
        let service = Arc::new(Service {
            state: Arc::new(State {
                stop: AtomicBool::new(false),
                snapshot: Mutex::new((
                    AudioSnapshot {
                        spectrum: [0.5; BANDS * 2],
                        level: [0.5; 2],
                        diagnostic: Some("waiting for PCM".into()),
                        ..AudioSnapshot::default()
                    },
                    Instant::now() - STALE - Duration::from_millis(1),
                )),
            }),
            worker: None,
        });
        CONSUMERS.fetch_add(1, Ordering::Relaxed);
        let snapshot = AudioConsumer(service).snapshot();
        assert_eq!(snapshot.spectrum, [0.0; BANDS * 2]);
        assert_eq!(snapshot.level, [0.0; 2]);
        assert_eq!(snapshot.diagnostic.as_deref(), Some("waiting for PCM"));
    }
    #[test]
    fn real_playback_monitor_if_requested() {
        if std::env::var_os("VARPAPER_TEST_PULSE").is_none() {
            return;
        }
        let baseline = live_counts();
        let consumer = AudioConsumer::acquire();
        let deadline = Instant::now() + Duration::from_secs(5);
        loop {
            let snapshot = consumer.snapshot();
            if let Some(monitor) = snapshot.monitor.filter(|_| snapshot.sequence > 0) {
                assert!(snapshot.diagnostic.is_none());
                assert!(snapshot.spectrum.iter().all(|v| v.is_finite()));
                println!("Playback monitor connected: {monitor}");
                break;
            }
            assert!(Instant::now() < deadline, "{:?}", snapshot.diagnostic);
            std::thread::sleep(Duration::from_millis(10));
        }
        drop(consumer);
        assert_eq!(live_counts(), baseline);
    }
}
