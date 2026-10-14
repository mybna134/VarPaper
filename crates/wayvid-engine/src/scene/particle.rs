//! Bounded, assignment-local particle simulation with an injectable random seed.
//! Rendering and the remaining reference operators are qualified separately.
use super::model::{bound, scalar, vector};
use anyhow::{ensure, Context, Result};
use glam::Vec3;
use serde_json::Value;

#[derive(Clone, Debug)]
pub struct Definition {
    pub material: String,
    pub capacity: usize,
    emitters: Vec<Emitter>,
    initializers: Vec<Initializer>,
    operators: Vec<Operator>,
}
#[derive(Clone, Debug)]
struct Emitter {
    sphere: bool,
    origin: Vec3,
    minimum: Vec3,
    maximum: Vec3,
    rate: f32,
    burst: usize,
    delay: f32,
    duration: f32,
    directions: Vec3,
    sign: Vec3,
    speed: [f32; 2],
}
#[derive(Clone, Debug)]
enum Initializer {
    Color(Vec3, Vec3),
    Size(f32, f32, f32),
    Alpha(f32, f32),
    Lifetime(f32, f32),
    Velocity(Vec3, Vec3),
    Rotation(Vec3, Vec3),
    AngularVelocity(Vec3, Vec3),
}
#[derive(Clone, Debug)]
enum Operator {
    Movement(f32, Vec3),
    AngularMovement(f32, Vec3),
    AlphaFade(f32, f32),
    SizeChange(f32, f32, f32, f32),
    AlphaChange(f32, f32, f32, f32),
    ColorChange(f32, f32, Vec3, Vec3),
}
fn number(value: &Value, key: &str, default: f32) -> Result<f32> {
    let result = scalar(&value[key], default)?;
    ensure!(
        result.abs() <= 1e6,
        "Particle field '{key}' exceeds numeric budget"
    );
    Ok(result)
}
fn vec(value: &Value, key: &str, default: Vec3) -> Result<Vec3> {
    let v = bound(&value[key]);
    let result = if let Some(value) = v.as_f64() {
        Vec3::splat(value as f32)
    } else {
        Vec3::from_array(vector(v, default.to_array())?)
    };
    ensure!(
        result.is_finite() && result.abs().max_element() <= 1e6,
        "Invalid particle vector '{key}'"
    );
    Ok(result)
}
fn list<'a>(value: &'a Value, key: &str) -> Result<&'a [Value]> {
    let Some(value) = value.get(key) else {
        return Ok(&[]);
    };
    let values = value
        .as_array()
        .with_context(|| format!("Particle '{key}' must be an array"))?;
    ensure!(values.len() <= 256, "Particle '{key}' count exceeds 256");
    ensure!(
        values.iter().all(Value::is_object),
        "Particle '{key}' contains an invalid entry"
    );
    Ok(values)
}
fn range(value: &Value, low: f32, high: f32) -> Result<(f32, f32)> {
    let (low, high) = (number(value, "min", low)?, number(value, "max", high)?);
    ensure!(low <= high, "Reversed particle random range");
    Ok((low, high))
}
fn vector_range(value: &Value, low: Vec3, high: Vec3) -> Result<(Vec3, Vec3)> {
    let (low, high) = (vec(value, "min", low)?, vec(value, "max", high)?);
    ensure!(low.cmple(high).all(), "Reversed particle vector range");
    Ok((low, high))
}
impl Definition {
    pub fn parse(value: &Value) -> Result<Self> {
        ensure!(value.is_object(), "Particle definition must be an object");
        let material = value["material"]
            .as_str()
            .context("Particle must have a material")?
            .to_string();
        wayvid_library::project::resource_name(&material)?;
        let capacity = value
            .get("maxcount")
            .map(|v| v.as_u64().context("Invalid particle maxcount"))
            .transpose()?
            .unwrap_or(100);
        ensure!(
            (1..=16384).contains(&capacity),
            "Particle maxcount exceeds 1..16384"
        );
        // These features must not be silently discarded while being migrated.
        for field in ["children", "controlpoint"] {
            ensure!(
                list(value, field)?.is_empty(),
                "Rust particle {field} integration is incomplete"
            );
        }
        for renderer in list(value, "renderer")? {
            ensure!(
                renderer["name"].as_str().unwrap_or("sprite") == "sprite",
                "Rust particle renderer integration is incomplete: {}",
                renderer["name"]
            );
        }
        ensure!(
            value.get("flags").is_none_or(|v| v.as_u64() == Some(0)),
            "Rust particle perspective integration is incomplete"
        );
        let mut emitters = Vec::new();
        for value in list(value, "emitter")? {
            let name = value["name"]
                .as_str()
                .context("Particle emitter must have a name")?;
            ensure!(
                ["boxrandom", "sphererandom"].contains(&name),
                "Unsupported required particle emitter '{name}'"
            );
            ensure!(
                value.get("flags").is_none_or(|v| v.as_u64() == Some(0)),
                "Rust particle emitter flags integration is incomplete"
            );
            let rate = number(value, "rate", 10.0)?;
            let delay = number(value, "delay", 0.0)?;
            let duration = number(value, "duration", 0.0)?;
            ensure!(
                rate >= 0.0 && delay >= 0.0 && duration >= 0.0,
                "Negative particle emission parameter"
            );
            let burst = value
                .get("instantaneous")
                .map(|v| v.as_u64().context("Invalid particle instantaneous count"))
                .transpose()?
                .unwrap_or(0);
            ensure!(burst <= capacity, "Particle burst exceeds capacity");
            let minimum = vec(value, "distancemin", Vec3::ZERO)?;
            let maximum = vec(value, "distancemax", Vec3::new(256.0, 256.0, 0.0))?;
            ensure!(
                minimum.cmple(maximum).all(),
                "Reversed particle emission bounds"
            );
            let speed = [
                number(value, "speedmin", 0.0)?,
                number(value, "speedmax", 0.0)?,
            ];
            ensure!(speed[0] <= speed[1], "Reversed particle speed range");
            let sign = vec(value, "sign", Vec3::ZERO)?;
            ensure!(
                sign.to_array().iter().all(|v| [-1.0, 0.0, 1.0].contains(v)),
                "Invalid particle emitter sign"
            );
            ensure!(
                value
                    .get("controlpoint")
                    .is_none_or(|v| v.as_i64() == Some(0)),
                "Rust particle control point integration is incomplete"
            );
            ensure!(
                number(value, "cone", 0.0)? == 0.0,
                "Rust particle cone integration is incomplete"
            );
            emitters.push(Emitter {
                sphere: name == "sphererandom",
                origin: vec(value, "origin", Vec3::ZERO)?,
                minimum,
                maximum,
                rate,
                burst: burst as usize,
                delay,
                duration,
                directions: vec(value, "directions", Vec3::new(1.0, 1.0, 0.0))?,
                sign,
                speed,
            });
        }
        let mut initializers = Vec::new();
        for value in list(value, "initializer")? {
            let name = value["name"]
                .as_str()
                .context("Particle initializer must have a name")?;
            initializers.push(match name {
                "colorrandom" => {
                    let (a, b) = vector_range(value, Vec3::ZERO, Vec3::ONE)?;
                    ensure!(
                        a.min_element() >= 0.0 && b.max_element() <= 1.0,
                        "Particle colors must be normalized"
                    );
                    Initializer::Color(a, b)
                }
                "sizerandom" => {
                    let (a, b) = range(value, 0.0, 20.0)?;
                    let exponent = number(value, "exponent", 1.0)?;
                    ensure!(a >= 0.0 && exponent > 0.0, "Invalid particle size range");
                    Initializer::Size(a, b, exponent)
                }
                "alpharandom" => {
                    let (a, b) = range(value, 0.05, 1.0)?;
                    ensure!(a >= 0.0 && b <= 1.0, "Invalid particle alpha range");
                    Initializer::Alpha(a, b)
                }
                "lifetimerandom" => {
                    let (a, b) = range(value, 0.0, 1.0)?;
                    ensure!(a >= 0.0 && b > 0.0, "Invalid particle lifetime range");
                    Initializer::Lifetime(a, b)
                }
                "velocityrandom" => {
                    let (a, b) = vector_range(value, Vec3::splat(-32.0), Vec3::splat(32.0))?;
                    Initializer::Velocity(a, b)
                }
                "rotationrandom" => {
                    let (a, b) = vector_range(
                        value,
                        Vec3::ZERO,
                        Vec3::new(0.0, 0.0, std::f32::consts::TAU),
                    )?;
                    Initializer::Rotation(a, b)
                }
                "angularvelocityrandom" => {
                    let (a, b) =
                        vector_range(value, Vec3::new(0.0, 0.0, -5.0), Vec3::new(0.0, 0.0, 5.0))?;
                    Initializer::AngularVelocity(a, b)
                }
                _ => anyhow::bail!("Rust particle initializer integration is incomplete: '{name}'"),
            });
        }
        let mut operators = Vec::new();
        for value in list(value, "operator")? {
            let name = value["name"]
                .as_str()
                .context("Particle operator must have a name")?;
            operators.push(match name {
                "movement" | "angularmovement" => {
                    let drag = number(value, "drag", 0.0)?;
                    ensure!(drag >= 0.0, "Negative particle drag");
                    if name == "movement" {
                        Operator::Movement(drag, vec(value, "gravity", Vec3::ZERO)?)
                    } else {
                        Operator::AngularMovement(drag, vec(value, "force", Vec3::ZERO)?)
                    }
                }
                "alphafade" => {
                    let a = number(value, "fadeintime", 0.5)?;
                    let b = number(value, "fadeouttime", 0.5)?;
                    ensure!(a >= 0.0 && b >= 0.0, "Invalid particle alpha fade");
                    Operator::AlphaFade(a, b)
                }
                "sizechange" | "alphachange" | "colorchange" => {
                    let a = number(value, "starttime", 0.0)?;
                    let b = number(value, "endtime", 1.0)?;
                    ensure!(
                        a >= 0.0 && b >= a && b <= 1.0,
                        "Invalid particle change interval"
                    );
                    if name == "colorchange" {
                        Operator::ColorChange(
                            a,
                            b,
                            vec(value, "startvalue", Vec3::ONE)?,
                            vec(value, "endvalue", Vec3::ONE)?,
                        )
                    } else {
                        let start = number(value, "startvalue", 1.0)?;
                        let end = number(value, "endvalue", 0.0)?;
                        if name == "sizechange" {
                            Operator::SizeChange(a, b, start, end)
                        } else {
                            Operator::AlphaChange(a, b, start, end)
                        }
                    }
                }
                _ => anyhow::bail!("Rust particle operator integration is incomplete: '{name}'"),
            });
        }
        Ok(Self {
            material,
            capacity: capacity as usize,
            emitters,
            initializers,
            operators,
        })
    }
}
#[derive(Debug, Clone, PartialEq)]
pub struct Particle {
    pub position: Vec3,
    pub velocity: Vec3,
    pub rotation: Vec3,
    angular_velocity: Vec3,
    pub size: f32,
    pub color: Vec3,
    pub alpha: f32,
    pub age: f32,
    pub lifetime: f32,
    initial_size: f32,
    initial_alpha: f32,
    initial_color: Vec3,
}
#[derive(Default)]
struct EmitterState {
    credit: f64,
    burst_sent: bool,
}
pub struct Simulation {
    definition: Definition,
    pub particles: Vec<Particle>,
    emitters: Vec<EmitterState>,
    random: u64,
    elapsed: f64,
}
fn random(state: &mut u64) -> f32 {
    // SplitMix64 provides a fully specified, reproducible stream, including seed 0.
    *state = state.wrapping_add(0x9e3779b97f4a7c15);
    let mut z = *state;
    z = (z ^ (z >> 30)).wrapping_mul(0xbf58476d1ce4e5b9);
    z = (z ^ (z >> 27)).wrapping_mul(0x94d049bb133111eb);
    ((z ^ (z >> 31)) >> 40) as f32 / 16777216.0
}
fn random_vec(state: &mut u64, a: Vec3, b: Vec3) -> Vec3 {
    a + (b - a) * Vec3::new(random(state), random(state), random(state))
}
fn phase(age: f32, life: f32, a: f32, b: f32) -> f32 {
    if b == a {
        if age / life >= b {
            1.0
        } else {
            0.0
        }
    } else {
        ((age / life - a) / (b - a)).clamp(0.0, 1.0)
    }
}
impl Simulation {
    pub fn new(definition: Definition, seed: u64) -> Self {
        let emitters = (0..definition.emitters.len())
            .map(|_| EmitterState::default())
            .collect();
        Self {
            particles: Vec::with_capacity(definition.capacity),
            definition,
            emitters,
            random: seed,
            elapsed: 0.0,
        }
    }
    pub fn advance(&mut self, delta: f64) -> Result<()> {
        ensure!(
            delta.is_finite() && (0.0..=0.25).contains(&delta),
            "Invalid particle timestep"
        );
        if delta == 0.0 {
            return Ok(());
        }
        self.elapsed += delta;
        let dt = delta as f32;
        for particle in &mut self.particles {
            particle.age += dt;
            particle.alpha = particle.initial_alpha;
            particle.size = particle.initial_size;
            particle.color = particle.initial_color;
            for operator in &self.definition.operators {
                match *operator {
                    Operator::Movement(drag, gravity) => {
                        particle.position += particle.velocity * dt;
                        particle.velocity =
                            (particle.velocity + gravity * dt) * (1.0 - drag * dt).max(0.0);
                    }
                    Operator::AngularMovement(drag, force) => {
                        particle.rotation += particle.angular_velocity * dt;
                        particle.angular_velocity =
                            (particle.angular_velocity + force * dt) * (1.0 - drag * dt).max(0.0);
                    }
                    Operator::AlphaFade(a, b) => {
                        let life = particle.age / particle.lifetime;
                        let fade = if life <= a {
                            if a > 0.0 {
                                life / a
                            } else {
                                1.0
                            }
                        } else if life > b {
                            if b < 1.0 {
                                (1.0 - life) / (1.0 - b)
                            } else {
                                1.0
                            }
                        } else {
                            1.0
                        };
                        particle.alpha *= fade.clamp(0.0, 1.0);
                    }
                    Operator::SizeChange(a, b, start, end) => {
                        particle.size = particle.initial_size
                            * (start
                                + (end - start) * phase(particle.age, particle.lifetime, a, b));
                    }
                    Operator::AlphaChange(a, b, start, end) => {
                        particle.alpha = particle.initial_alpha
                            * (start
                                + (end - start) * phase(particle.age, particle.lifetime, a, b));
                    }
                    Operator::ColorChange(a, b, start, end) => {
                        particle.color = particle.initial_color
                            * start.lerp(end, phase(particle.age, particle.lifetime, a, b));
                    }
                }
            }
        }
        self.particles
            .retain(|particle| particle.age < particle.lifetime);
        for (emitter, state) in self.definition.emitters.iter().zip(&mut self.emitters) {
            let start = self.elapsed - delta;
            let active_end = if emitter.duration > 0.0 {
                (f64::from(emitter.delay) + f64::from(emitter.duration)).min(self.elapsed)
            } else {
                self.elapsed
            };
            let active = (active_end - start.max(f64::from(emitter.delay))).max(0.0);
            if active == 0.0 {
                continue;
            }
            state.credit += active * f64::from(emitter.rate);
            let mut count = (state.credit.floor() as usize).min(self.definition.capacity);
            state.credit -= state.credit.floor();
            if !state.burst_sent {
                count = count.saturating_add(emitter.burst);
                state.burst_sent = true;
            }
            count = count.min(self.definition.capacity - self.particles.len());
            for _ in 0..count {
                let mut offset = if emitter.sphere {
                    let angle = std::f32::consts::TAU * random(&mut self.random);
                    let (low, high) = (emitter.minimum.x, emitter.maximum.x);
                    let radius =
                        (low * low + (high * high - low * low) * random(&mut self.random)).sqrt();
                    Vec3::new(
                        radius * angle.cos(),
                        radius * angle.sin(),
                        (2.0 * random(&mut self.random) - 1.0) * high,
                    ) * emitter.directions
                } else {
                    let mut offset = random_vec(&mut self.random, emitter.minimum, emitter.maximum);
                    for value in offset.as_mut() {
                        if random(&mut self.random) < 0.5 {
                            *value = -*value;
                        }
                    }
                    offset * emitter.directions
                };
                if emitter.sphere {
                    for axis in 0..3 {
                        if emitter.sign[axis] != 0.0 {
                            offset[axis] = offset[axis].abs() * emitter.sign[axis];
                        }
                    }
                }
                let velocity = if emitter.sphere && emitter.speed != [0.0; 2] {
                    let direction = offset.try_normalize().unwrap_or(Vec3::Y);
                    direction
                        * (emitter.speed[0]
                            + (emitter.speed[1] - emitter.speed[0]) * random(&mut self.random))
                } else {
                    Vec3::ZERO
                };
                let mut p = Particle {
                    position: emitter.origin + offset,
                    velocity,
                    rotation: Vec3::ZERO,
                    angular_velocity: Vec3::ZERO,
                    size: 20.0,
                    color: Vec3::ONE,
                    alpha: 1.0,
                    age: 0.0,
                    lifetime: 1.0,
                    initial_size: 20.0,
                    initial_alpha: 1.0,
                    initial_color: Vec3::ONE,
                };
                for init in &self.definition.initializers {
                    match *init {
                        Initializer::Color(a, b) => p.color = random_vec(&mut self.random, a, b),
                        Initializer::Size(a, b, exp) => {
                            p.size = a + (b - a) * random(&mut self.random).powf(exp)
                        }
                        Initializer::Alpha(a, b) => {
                            p.alpha = a + (b - a) * random(&mut self.random)
                        }
                        Initializer::Lifetime(a, b) => {
                            p.lifetime = (a + (b - a) * random(&mut self.random)).max(f32::EPSILON)
                        }
                        Initializer::Velocity(a, b) => {
                            p.velocity = random_vec(&mut self.random, a, b)
                        }
                        Initializer::Rotation(a, b) => {
                            p.rotation = random_vec(&mut self.random, a, b)
                        }
                        Initializer::AngularVelocity(a, b) => {
                            p.angular_velocity = random_vec(&mut self.random, a, b)
                        }
                    }
                }
                p.initial_size = p.size;
                p.initial_alpha = p.alpha;
                p.initial_color = p.color;
                self.particles.push(p);
            }
        }
        ensure!(
            self.particles.iter().all(|p| p.position.is_finite()
                && p.velocity.is_finite()
                && p.rotation.is_finite()
                && p.color.is_finite()
                && p.size.is_finite()
                && p.alpha.is_finite()),
            "Particle simulation overflow"
        );
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    fn fixture() -> Value {
        serde_json::json!({"material":"material.json","maxcount":16,"emitter":[{"name":"boxrandom","distancemin":0,"distancemax":2,"rate":0,"instantaneous":4}],"initializer":[{"name":"velocityrandom","min":[1,0,0],"max":[3,0,0]},{"name":"lifetimerandom","min":0.5,"max":0.5}],"operator":[{"name":"movement","gravity":[0,1,0]}]})
    }
    #[test]
    fn seeded_particles_are_deterministic_bounded_and_advance() {
        let definition = Definition::parse(&fixture()).unwrap();
        let mut a = Simulation::new(definition.clone(), 7);
        let mut b = Simulation::new(definition.clone(), 7);
        let mut c = Simulation::new(definition, 8);
        a.advance(0.1).unwrap();
        b.advance(0.1).unwrap();
        c.advance(0.1).unwrap();
        assert_eq!(a.particles.len(), 4);
        assert_eq!(a.particles, b.particles);
        assert_ne!(a.particles, c.particles);
        let frozen = a.particles.clone();
        a.advance(0.0).unwrap();
        assert_eq!(a.particles, frozen);
        a.advance(0.1).unwrap();
        assert_ne!(a.particles, frozen);
        assert!(a.advance(f64::NAN).is_err());
        assert!(a.advance(1.0).is_err());
        a.advance(0.25).unwrap();
        a.advance(0.25).unwrap();
        assert!(a.particles.is_empty());
    }
    #[test]
    fn bad_particle_modes_budgets_and_numeric_fields_are_diagnosed() {
        let mut value = fixture();
        value["maxcount"] = serde_json::json!(16385);
        assert!(Definition::parse(&value).is_err());
        let mut value = fixture();
        value["emitter"][0]["rate"] = serde_json::json!(-1);
        assert!(Definition::parse(&value).is_err());
        let mut value = fixture();
        value["operator"][0]["name"] = serde_json::json!("unknown");
        assert!(Definition::parse(&value)
            .unwrap_err()
            .to_string()
            .contains("unknown"));

        let mut value = fixture();
        value["renderer"] = serde_json::json!([{ "name": "rope" }]);
        assert!(Definition::parse(&value)
            .unwrap_err()
            .to_string()
            .contains("renderer"));
        let mut value = fixture();
        value["controlpoint"] = serde_json::json!([{}]);
        assert!(Definition::parse(&value)
            .unwrap_err()
            .to_string()
            .contains("controlpoint"));
        let mut value = fixture();
        value["initializer"] = serde_json::json!([{ "name": "turbulentvelocityrandom" }]);
        assert!(Definition::parse(&value)
            .unwrap_err()
            .to_string()
            .contains("initializer"));
        let mut value = fixture();
        value["operator"] = serde_json::json!([{ "name": "vortex" }]);
        assert!(Definition::parse(&value)
            .unwrap_err()
            .to_string()
            .contains("operator"));
    }
}
