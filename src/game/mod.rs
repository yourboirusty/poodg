mod collisions;
mod object;
mod pudge;
mod spawner;

use embedded_graphics::image::Image;
use embedded_graphics::prelude::*;
use embedded_graphics::{geometry::Point, text::Text};
use fugit::TimerInstantU64;
use nalgebra::Vector2;
use object::{GameObject, GameObjectSignal, ObjectHandler};
use pudge::{Pudge, PudgeSignal};
use spawner::Spawner;

use crate::graphics::image::{draw_hp, draw_image, draw_text};
use crate::graphics::resources::SPLASH;
use crate::{
    controls::ControlEnum,
    graphics::{display::DisplayEnum, resources::TEXT_STYLE},
};
#[cfg(target_os = "none")]
use libm::floorf;

#[cfg(target_arch = "wasm32")]
const TICK_RATE: u64 = 1000;
#[cfg(not(target_arch = "wasm32"))]
const TICK_RATE: u64 = 16_000;

type Id = u16;
type TickCount = u32;
type Speed = f32;

pub const CLOCK_HZ: u32 = 1_000_000;
pub type Instant = TimerInstantU64<CLOCK_HZ>;

#[derive(Debug, Clone, Copy, PartialEq)]
pub struct GamePoint {
    x: f32,
    y: f32,
}

impl GamePoint {
    pub fn new(x: i32, y: i32) -> Self {
        Self {
            x: x as f32,
            y: y as f32,
        }
    }

    pub fn in_rect(&self, rect_corner: &GamePoint, width: &u8, height: &u8) -> bool {
        self.x >= rect_corner.x
            && self.x <= rect_corner.x + *width as f32
            && self.y <= rect_corner.y
            && self.y >= rect_corner.y - *height as f32
    }
}
impl Into<Vector2<f32>> for GamePoint {
    fn into(self) -> Vector2<f32> {
        return Vector2::new(self.x, self.y);
    }
}
impl Into<Point> for GamePoint {
    fn into(self) -> Point {
        #[cfg(not(target_os = "none"))]
        {
            return Point {
                x: self.x.floor() as i32,
                y: self.y.floor() as i32,
            };
        }
        #[cfg(target_os = "none")]
        {
            return Point {
                x: floorf(self.x) as i32,
                y: floorf(self.y) as i32,
            };
        }
    }
}

#[derive(Debug, PartialEq, Clone, Copy)]
enum RuneEnum {
    WalkSpeed(i32),
    HookSpeed(i32),
    HookSize(i32),
}

#[derive(Debug, PartialEq, Clone, Copy)]
pub enum Creep {
    Radiant,
    Dire,
}

#[derive(Debug, PartialEq, Clone, Copy)]
enum Hookable {
    Creep(Creep),
    Blockade,
    Rune(RuneEnum),
}

enum GameDifficultyEnum {
    Easy,
    Medium,
    Hard,
    Dendi,
}

pub struct Game {
    state: GameState,
    time: Option<Instant>,
    seed: u64,
    pudge: Pudge,
    object_handler: ObjectHandler,
    spawner: Option<Spawner>,
}

type Score = i32;
type Hp = u8;

type Started = bool;

#[derive(Clone, Copy, PartialEq, Eq)]
enum GameState {
    Init(Started),
    Hookin(Score, Hp),
    GameOver(Started, Score),
}

impl GameState {
    pub fn next(&mut self) {
        match self.clone() {
            GameState::Init(_) => *self = GameState::Hookin(0, 3),
            GameState::Hookin(score, _) => *self = GameState::GameOver(false, score.clone()),
            GameState::GameOver(_, _) => *self = GameState::Hookin(0, 3),
        }
    }
    pub fn set_started(&mut self) {
        match self.clone() {
            GameState::Init(false) => *self = GameState::Init(true),
            GameState::GameOver(false, score) => *self = GameState::GameOver(true, score),
            _ => {}
        }
    }
    pub fn add_score(&mut self, points: i16) {
        if let GameState::Hookin(score, hp) = self.clone() {
            if points < 0 && (-points) as i32 > score {
                *self = GameState::Hookin(0, hp);
                return;
            }
            *self = GameState::Hookin(score + points as i32, hp);
        } else {
            panic!("Can't add score to non-active game");
        }
    }

    pub fn damage(&mut self) {
        if let GameState::Hookin(score, hp) = self.clone() {
            if hp == 1 {
                self.next();
                return;
            }
            *self = GameState::Hookin(score, hp - 1);
        } else {
            panic!("Can't damage non-active game");
        }
    }

    pub fn to_str(&self) -> &str {
        match self {
            GameState::GameOver(_, _) => "Game Over",
            GameState::Init(_) => "Game start",
            GameState::Hookin(_, _) => "Game in progress",
        }
    }
}

impl Default for Game {
    fn default() -> Self {
        return Game {
            state: GameState::Init(false),
            spawner: None,
            time: None,
            seed: 0,
            pudge: Pudge::default(),
            object_handler: ObjectHandler::new(),
        };
    }
}
impl Game {
    pub fn init(&mut self) {
        //TODO: difficulty scaling
        self.pudge = Pudge::default();
        self.spawner = Some(Spawner::new(1, 123489, 0.65));
    }

    pub fn set_seed(&mut self, seed: u64) {
        self.seed = seed;
    }

    pub fn process(&mut self, new_time: Instant) {
        if self.time.is_none() {
            self.time = Some(new_time.clone());
        }
        let delta = new_time.checked_duration_since(self.time.unwrap()).unwrap();
        self.time = Some(new_time);

        if delta.to_micros() < TICK_RATE {
            return;
        }

        match self.state {
            GameState::Init(_) => self.selector_tick(),
            GameState::Hookin(_, _) => self.main_tick(),
            GameState::GameOver(_, _) => self.selector_tick(),
        }
    }
    fn selector_tick(&mut self) {
        match self.state {
            GameState::Init(false) | GameState::GameOver(false, _) => self.insert_selector(),
            _ => {}
        }

        let pudge_signal = self.pudge.tick(&mut self.object_handler);
        if let Some(PudgeSignal::Reeled(Some(_))) = pudge_signal {
            if let GameState::GameOver(_, _) = self.state {
                self.seed += 73_432;
                self.init();
            }
            self.state.next();
        }
    }

    fn insert_selector(&mut self) {
        let starter_creep = GameObject::make_creep(0, GamePoint::new(0, 24), Creep::Radiant, 0.0);
        self.object_handler.insert(starter_creep);
        self.state.set_started();
    }

    fn main_tick(&mut self) {
        let pudge_signal = self.pudge.tick(&mut self.object_handler);
        if let Some(signal) = pudge_signal {
            match signal {
                PudgeSignal::Hooked(obj) => {
                    if obj.game_type == Hookable::Creep(Creep::Radiant) {
                        let points = obj.calculate_score();
                        self.state
                            .add_score(points.try_into().expect("Points too large!"));
                    }
                }
                PudgeSignal::Missed => {
                    self.state.add_score(-10);
                }
                PudgeSignal::Reeled(_obj) => {
                    //todo
                }
            }
        }

        let mut to_remove: [Option<Id>; 8] = [None; 8];
        for object in self.object_handler.iter_mut() {
            let signal = object.tick();
            if let Some(GameObjectSignal::OutOfBounds(id)) = signal {
                for obj in to_remove.iter_mut() {
                    if obj.is_none() {
                        *obj = Some(id);
                        break;
                    }
                }
            }
        }
        for obj in to_remove.iter() {
            if let Some(id) = obj {
                if let Some(deleted_obj) = self.object_handler.remove(*id) {
                    if deleted_obj.game_type == Hookable::Creep(Creep::Radiant) {
                        self.state.damage();
                    }
                }
            } else {
                break;
            }
        }
        if let Some(ref mut spawn) = self.spawner {
            if let Some(object) = spawn.try_spawn(self.time.unwrap()) {
                self.object_handler.insert(object);
            }
        }
    }

    pub fn control(&mut self, controls: ControlEnum) {
        let _signal = self.pudge.act(Some(controls));
    }

    pub fn draw(&mut self, display: &mut DisplayEnum) {
        match self.state {
            GameState::Init(_) => {
                let splash = Image::new(&SPLASH, Point::zero());
                draw_image(display, splash);
            }
            GameState::Hookin(score, hp) => {
                let mut buffer = itoa::Buffer::new();
                let score_str = buffer.format(score);
                let score_text = Text::with_baseline(
                    score_str,
                    Point::new(2, 4),
                    TEXT_STYLE.clone(),
                    embedded_graphics::text::Baseline::Top,
                );
                draw_text(display, score_text);
                draw_hp(display, hp);
            }
            GameState::GameOver(_, score) => {
                let mut buffer = itoa::Buffer::new();
                let score_str = buffer.format(score);
                let score_text = Text::with_baseline(
                    score_str,
                    Point::new(64, 4),
                    TEXT_STYLE.clone(),
                    embedded_graphics::text::Baseline::Top,
                );
                draw_text(display, score_text);
                let game_over_text = Text::with_baseline(
                    "GAME OVER, TRY AGAIN?",
                    Point::new(36, 32),
                    TEXT_STYLE.clone(),
                    embedded_graphics::text::Baseline::Top,
                );
                draw_text(display, game_over_text);
            }
        }
        for object in self.object_handler.iter() {
            object.draw(display);
        }
        self.pudge.draw(display);
    }
}
