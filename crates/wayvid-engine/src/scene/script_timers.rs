//! Timer registration and scheduling live in Rust, in animation-clock seconds.
use rquickjs::{function::Opt, Ctx, Exception, Function, Object, Persistent, Result};
use std::{cell::RefCell, collections::BTreeMap, rc::Rc};
struct Timer {
    callback: Persistent<Function<'static>>,
    due: f64,
    interval: Option<f64>,
}
#[derive(Default)]
pub(super) struct Timers {
    next: u32,
    pub now: f64,
    entries: BTreeMap<u32, Timer>,
}
pub(super) type Shared = Rc<RefCell<Timers>>;
impl Timers {
    pub fn clear(&mut self) {
        self.entries.clear();
    }
    pub fn due(&self) -> Vec<u32> {
        self.entries
            .iter()
            .filter_map(|(id, timer)| (timer.due <= self.now).then_some(*id))
            .collect()
    }
    pub fn take(&mut self, id: u32) -> Option<Persistent<Function<'static>>> {
        let timer = self.entries.remove(&id)?;
        let callback = timer.callback.clone();
        if let Some(interval) = timer.interval {
            self.entries.insert(
                id,
                Timer {
                    due: self.now + interval,
                    ..timer
                },
            );
        }
        Some(callback)
    }
}
fn register<'js>(ctx: Ctx<'js>, shared: Shared, repeat: bool) -> Result<Function<'js>> {
    Function::new(
        ctx,
        move |ctx: Ctx<'js>, callback: Function<'js>, delay: Opt<f64>| -> Result<Function<'js>> {
            let delay = delay.0.unwrap_or(0.0);
            if !delay.is_finite() || !(0.0..=86400000.0).contains(&delay) {
                return Err(Exception::throw_range(&ctx, "Invalid Scene timer delay"));
            }
            let mut timers = shared.borrow_mut();
            if timers.entries.len() >= 1024 {
                return Err(Exception::throw_range(
                    &ctx,
                    "Scene timer count exceeds 1024",
                ));
            }
            let id = timers
                .next
                .checked_add(1)
                .ok_or_else(|| Exception::throw_range(&ctx, "Scene timer id exhausted"))?;
            timers.next = id;
            let interval = (delay / 1000.0).max(0.001);
            let due = timers.now + delay / 1000.0;
            timers.entries.insert(
                id,
                Timer {
                    callback: Persistent::save(&ctx, callback),
                    due,
                    interval: repeat.then_some(interval),
                },
            );
            drop(timers);
            let shared = shared.clone();
            Function::new(ctx, move || {
                shared.borrow_mut().entries.remove(&id);
            })
        },
    )
}
pub(super) fn install<'js>(ctx: Ctx<'js>, engine: &Object<'js>, shared: Shared) -> Result<()> {
    for (name, repeat) in [("setTimeout", false), ("setInterval", true)] {
        let function = register(ctx.clone(), shared.clone(), repeat)?;
        engine.set(name, function.clone())?;
        ctx.globals().set(name, function)?;
    }
    Ok(())
}
