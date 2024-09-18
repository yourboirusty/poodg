use super::{object::GameObject, GamePoint, Id, Instant, Speed};

use rand::prelude::*;

pub struct Spawner {
    spawn_rate_ms: u32,
    base_speed: Speed,
    rng: SmallRng,
    spawn_axes: [u8; 2],
    last_id: Id,
    last_spawn: Instant,
}

pub struct SpawnerBuilder {
    creep_per_m: u8,
    seed: u64,
    base_speed: Speed,
}

impl Spawner {
    pub fn from_config(config: SpawnerBuilder) -> Self {
        let spawn_rate_ms = 6000 / config.creep_per_m as u32;
        Self {
            spawn_rate_ms,
            base_speed: config.base_speed,
            rng: SmallRng::seed_from_u64(config.seed),
            spawn_axes: [28, 10],
            last_id: 0,
            last_spawn: Instant::from_ticks(0),
        }
    }
    pub fn new(creep_per_m: u8, seed: u64, base_speed: Speed) -> Self {
        let spawn_rate_ms = 6000 / creep_per_m as u32;
        Self {
            spawn_rate_ms,
            base_speed,
            rng: SmallRng::seed_from_u64(seed),
            spawn_axes: [28, 10],
            last_id: 0,
            last_spawn: Instant::from_ticks(0),
        }
    }

    fn max_spawn_deviation(&self) -> u32 {
        self.spawn_rate_ms / 3
    }

    fn spawn_check(&mut self, time: Instant) -> bool {
        let since_last_spawn = time.checked_duration_since(self.last_spawn).unwrap();
        let spawn_timer =
            since_last_spawn.to_millis() + self.rng.gen_range(0..self.max_spawn_deviation()) as u64;
        spawn_timer > self.spawn_rate_ms.into()
    }

    fn random_object(&mut self) -> GameObject {
        let id = self.last_id + 1;
        self.last_id = id;
        let upper_axes = self.rng.gen_bool(0.3);
        let go_right = self.rng.gen_bool(0.5);
        let y = if upper_axes {
            self.spawn_axes[1]
        } else {
            self.spawn_axes[0]
        };
        let x = if go_right { 0 } else { 128 };
        let speed = if go_right {
            self.base_speed + self.rng.gen_range(0..1) as f32
        } else {
            -self.base_speed - self.rng.gen_range(0..1) as f32
        };

        let location = GamePoint::new(x, y.into());

        //TODO: other creep types etc.
        let alliegiance = super::Creep::Radiant;

        GameObject::make_creep(id, location, alliegiance, speed)
    }

    pub fn try_spawn(&mut self, time: Instant) -> Option<GameObject> {
        if !self.spawn_check(time) {
            return None;
        }
        self.last_spawn = time;
        self.base_speed += 0.25;
        Some(self.random_object())
    }
}
