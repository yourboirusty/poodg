#[cfg(target_os = "none")]
extern crate alloc;

use crate::{
    controls::ControlEnum,
    graphics::{
        self,
        display::DisplayEnum,
        image::draw_image,
        resources::{HOOK_WIDTH, PUDGE_HEIGHT, PUDGE_WIDTH, SCREEN_WIDTH},
    },
    SpriteImage,
};

use super::{
    collisions::CollisionRectangle,
    object::{GameObject, ObjectHandler},
    GamePoint, Hookable, Speed, TickCount,
};
use embedded_graphics::{geometry::Point, image::Image};
use nalgebra::Vector2;

#[derive(Debug, PartialEq)]
enum HookState {
    Flying,
    Reeling(Option<Hookable>),
}

#[derive(Debug, PartialEq)]
enum PudgeState {
    Walking,
    Hooking(GamePoint, HookState),
    Cooldown(TickCount),
}

#[derive(Debug)]
pub enum PudgeSignal {
    Hooked(GameObject),
    Missed,
    Reeled(Option<Hookable>),
}

pub struct Pudge {
    location: GamePoint,
    speed: Speed,
    hook_speed: Speed,
    hook_boost: Speed,
    movement_boost: Speed,
    state: PudgeState,
}

impl Default for Pudge {
    fn default() -> Self {
        return Pudge {
            location: GamePoint::new(64, 64 - 13),
            speed: 2.75,
            hook_speed: 2.0,
            hook_boost: 0.1,
            movement_boost: 0.15,
            state: PudgeState::Walking,
        };
    }
}

impl Pudge {
    pub fn act(&mut self, controls: Option<ControlEnum>) {
        if self.state == PudgeState::Walking {
            if let Some(control) = controls {
                self.move_character(control);
            }
        }
    }

    pub fn tick(&mut self, object_handler: &mut ObjectHandler) -> Option<PudgeSignal> {
        if let PudgeState::Hooking(hook_location, hook_state) = &mut self.state {
            match hook_state {
                HookState::Flying => {
                    let mut new_location = hook_location.clone();

                    new_location.y -= self.hook_speed;

                    if new_location.y <= 0.0 {
                        self.set_state(PudgeState::Hooking(new_location, HookState::Reeling(None)));
                        return Some(PudgeSignal::Missed);
                    }
                    let hook_box_location = Vector2::new(new_location.x, new_location.y + 5.0);
                    let hook_box = CollisionRectangle::new(
                        hook_box_location,
                        Vector2::new(HOOK_WIDTH.into(), 3.0),
                    );

                    if let Some(collision) = object_handler.get_collision(&hook_box) {
                        let obj = object_handler.remove(collision.id).unwrap();
                        self.set_state(PudgeState::Hooking(
                            new_location,
                            HookState::Reeling(Some(obj.game_type)),
                        ));
                        return Some(PudgeSignal::Hooked(obj));
                    }
                    self.set_state(PudgeState::Hooking(new_location, HookState::Flying))
                }
                HookState::Reeling(obj) => {
                    let mut new_location = hook_location.clone();
                    let obj_clone = obj.clone();
                    new_location.y += self.hook_speed * 1.2;
                    if new_location.y >= self.location.y - PUDGE_HEIGHT as f32 {
                        self.set_state(PudgeState::Walking);
                        self.hook_speed += self.hook_boost;
                        self.speed += self.movement_boost;
                        return Some(PudgeSignal::Reeled(obj_clone));
                    }
                    self.set_state(PudgeState::Hooking(
                        new_location,
                        HookState::Reeling(obj_clone),
                    ))
                }
            }
        }
        None
    }
    fn set_state(&mut self, state: PudgeState) {
        self.state = state;
    }

    fn move_character(&mut self, controls: ControlEnum) {
        match controls {
            ControlEnum::None => return,
            ControlEnum::Left => {
                let distance = self.speed;
                if distance > self.location.x {
                    self.location.x = 0.0;
                    return;
                }
                self.location.x -= distance;
            }
            ControlEnum::Right => {
                let distance = self.speed;
                if distance + self.location.x + PUDGE_WIDTH as f32 > SCREEN_WIDTH as f32 {
                    self.location.x = (SCREEN_WIDTH as i16 - PUDGE_WIDTH as i16) as f32;
                    return;
                }
                self.location.x += distance;
            }
            ControlEnum::Hook => {
                let mut hook_location = self.location.clone();
                hook_location.y += PUDGE_HEIGHT as f32;
                self.state = PudgeState::Hooking(self.location.clone(), HookState::Flying)
            }
        }
    }

    pub fn draw(&self, display: &mut DisplayEnum) {
        self.draw_pudge(display);
        if let PudgeState::Hooking(position, hook_state) = &self.state {
            self.draw_hook(display, position, hook_state)
        }
    }

    fn draw_hook(&self, display: &mut DisplayEnum, position: &GamePoint, hook_state: &HookState) {
        let hook_pos: Point = (*position).into();
        let hook_img = Pudge::get_hook_img(&hook_pos, hook_state);
        draw_image(display, hook_img);
    }

    fn get_hook_img(position: &Point, hook_state: &HookState) -> SpriteImage {
        match hook_state {
            _ => Image::new(&graphics::resources::PUDGE_HOOK, *position),
        }
    }

    fn draw_pudge(&self, display: &mut DisplayEnum) {
        let image = Image::new(&graphics::resources::PUDGE_BODY, self.location.into());
        draw_image(display, image);
    }
}
