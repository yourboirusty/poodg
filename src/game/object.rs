#[cfg(not(target_os = "none"))]
use std::collections::{
    btree_map::{Values, ValuesMut},
    BTreeMap,
};

#[cfg(target_os = "none")]
extern crate alloc;
#[cfg(target_os = "none")]
use alloc::collections::{
    btree_map::{Values, ValuesMut},
    BTreeMap,
};

use embedded_graphics::image::Image;
use nalgebra::Vector2;

use crate::graphics::{
    display::{DisplayEnum, DISPLAY_HEIGHT},
    image::draw_image,
    resources::{Sprite, CREEP_HEIGHT, CREEP_WIDTH, SCREEN_WIDTH},
};

use super::{collisions::CollisionRectangle, Creep, GamePoint, Hookable, Id, Speed};

#[derive(Debug)]
pub struct GameObject {
    pub id: Id,
    location: GamePoint,
    pub game_type: Hookable,
    sprite_type: Sprite,
    width: u8,
    height: u8,
    reward: u16,
    speed: Speed,
}

pub struct ObjectHandler {
    objects: BTreeMap<Id, GameObject>,
}

impl GameObject {
    pub fn make_creep(id: Id, location: GamePoint, alliegiance: Creep, speed: Speed) -> Self {
        GameObject {
            id,
            location,
            game_type: Hookable::Creep(alliegiance),
            reward: 100,
            speed,
            sprite_type: Sprite::Creep(alliegiance),
            width: CREEP_WIDTH,
            height: CREEP_HEIGHT,
        }
    }

    pub fn calculate_score(&self) -> u32 {
        if self.game_type != Hookable::Creep(Creep::Radiant) {
            return 0;
        }
        self.reward as u32 + (self.speed + (DISPLAY_HEIGHT as f32 - self.location.y)) as u32
    }
}

pub enum GameObjectSignal {
    OutOfBounds(Id),
}

impl GameObject {
    pub fn tick(&mut self) -> Option<GameObjectSignal> {
        self.location.x += self.speed;
        if self.reward > 1 {
            self.reward -= 1;
        }
        if self.location.x < 0.0 || self.location.x > SCREEN_WIDTH as f32 {
            return Some(GameObjectSignal::OutOfBounds(self.id));
        }
        None
    }
    pub fn draw(&self, display: &mut DisplayEnum) {
        let ref img_raw = self.sprite_type.get_image();
        let img = Image::new(img_raw, self.location.into());
        draw_image(display, img);
    }

    pub fn contains(&self, point: &GamePoint) -> bool {
        return point.in_rect(&self.location, &self.width, &self.height);
    }

    pub fn intersects(&self, other_rect: &CollisionRectangle) -> bool {
        let collision_rect = CollisionRectangle::new(
            self.location.into(),
            Vector2::new(self.width.into(), self.height.into()),
        );
        collision_rect.intersects(other_rect)
    }
}

impl ObjectHandler {
    pub fn new() -> Self {
        ObjectHandler {
            objects: BTreeMap::new(),
        }
    }

    pub fn insert(&mut self, obj: GameObject) -> Option<GameObject> {
        self.objects.insert(obj.id, obj)
    }

    pub fn remove(&mut self, id: Id) -> Option<GameObject> {
        self.objects.remove(&id)
    }

    pub fn iter_mut(&mut self) -> ValuesMut<Id, GameObject> {
        self.objects.values_mut()
    }

    pub fn iter(&self) -> Values<u16, GameObject> {
        self.objects.values()
    }

    // Returns first collision
    pub fn get_collision(&self, rect: &CollisionRectangle) -> Option<&GameObject> {
        for obj in self.iter() {
            if obj.intersects(rect) {
                return Some(obj);
            }
        }
        None
    }
}
