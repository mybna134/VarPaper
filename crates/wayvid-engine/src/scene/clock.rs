//! Assignment-local animation time. Wall-clock/date inputs are separate.
use anyhow::{ensure, Result};

#[derive(Debug, Clone, Copy)]
pub struct FrameTime {
    pub elapsed: f64,
    pub delta: f64,
}

#[derive(Default, Debug, Clone)]
pub struct AnimationClock {
    last: Option<f64>,
    elapsed: f64,
    paused: bool,
}
impl AnimationClock {
    /// Consume a monotonic timestamp in seconds. Large frame gaps are bounded;
    /// callers may submit fixed smaller steps for deterministic simulation.
    pub fn tick(&mut self, monotonic: f64) -> Result<f64> {
        ensure!(
            monotonic.is_finite() && monotonic >= 0.0,
            "Invalid Scene monotonic time"
        );
        ensure!(
            self.last.is_none_or(|last| monotonic >= last),
            "Scene monotonic time moved backwards"
        );
        let delta = self
            .last
            .map(|last| (monotonic - last).min(0.25))
            .unwrap_or(0.0);
        self.last = Some(monotonic);
        let delta = if self.paused { 0.0 } else { delta };
        self.elapsed += delta;
        Ok(delta)
    }
    pub fn set_paused(&mut self, paused: bool, monotonic: f64) -> Result<()> {
        // An edge establishes a new baseline. No paused interval enters delta,
        // including when no frames were rendered during the pause.
        ensure!(
            monotonic.is_finite() && monotonic >= 0.0,
            "Invalid Scene monotonic time"
        );
        ensure!(
            self.last.is_none_or(|last| monotonic >= last),
            "Scene monotonic time moved backwards"
        );
        if paused != self.paused {
            self.last = Some(monotonic);
            self.paused = paused;
        }
        Ok(())
    }
    pub fn is_paused(&self) -> bool {
        self.paused
    }
    pub fn elapsed(&self) -> f64 {
        self.elapsed
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn injected_steps_pause_without_frames_resume_and_bad_input() {
        let mut clock = AnimationClock::default();
        assert_eq!(clock.tick(10.0).unwrap(), 0.0);
        assert!((clock.tick(10.1).unwrap() - 0.1).abs() < 1e-10);
        clock.set_paused(true, 10.1).unwrap();
        assert_eq!(clock.tick(20.0).unwrap(), 0.0);
        let frozen = clock.elapsed();
        clock.set_paused(false, 100.0).unwrap();
        assert!((clock.tick(100.02).unwrap() - 0.02).abs() < 1e-10);
        assert!((clock.elapsed() - frozen - 0.02).abs() < 1e-10);
        assert!(clock.tick(99.0).is_err());
        assert!(clock.tick(f64::NAN).is_err());
        assert_eq!(clock.tick(200.0).unwrap(), 0.25);
    }
}
