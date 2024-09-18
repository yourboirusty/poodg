use embedded_graphics::{
    image::ImageRaw,
    mono_font::{ascii, MonoTextStyle, MonoTextStyleBuilder},
    pixelcolor::BinaryColor,
    primitives::PrimitiveStyle,
};

use crate::game::Creep;

pub const SCREEN_WIDTH: u8 = 128;
pub const SCREEN_HEIGHT: u8 = 64;

pub const TEXT_STYLE: MonoTextStyle<BinaryColor> = MonoTextStyleBuilder::new()
    .font(&ascii::FONT_4X6)
    .text_color(BinaryColor::On)
    .build();
pub const WHITE_LINE: PrimitiveStyle<BinaryColor> = PrimitiveStyle::with_stroke(BinaryColor::On, 1);
pub const BLACK_LINE: PrimitiveStyle<BinaryColor> =
    PrimitiveStyle::with_stroke(BinaryColor::Off, 1);
pub const WHITE_FILL: PrimitiveStyle<BinaryColor> = PrimitiveStyle::with_fill(BinaryColor::On);
pub const BLACK_FILL: PrimitiveStyle<BinaryColor> = PrimitiveStyle::with_fill(BinaryColor::Off);
pub const CHAR_WIDTH: usize = 4;

// 18x13
pub const PUDGE_WIDTH: u8 = 18;
pub const PUDGE_HEIGHT: u8 = 13;
pub const PUDGE_BODY: ImageRaw<BinaryColor> =
    ImageRaw::new(include_bytes!("../../assets/PudgeBody.raw"), 18);
// 5x9
pub const HOOK_WIDTH: u8 = 5;
pub const HOOK_HEIGHT: u8 = 9;
pub const PUDGE_HOOK: ImageRaw<BinaryColor> = ImageRaw::new(
    include_bytes!("../../assets/PudgeHook.raw"),
    HOOK_WIDTH as u32,
);
// 7x3
pub const PUDGE_CLEAVER: ImageRaw<BinaryColor> =
    ImageRaw::new(include_bytes!("../../assets/PudgeCleaver.raw"), 3);
// 13x11
pub const CREEP_WIDTH: u8 = 13;
pub const CREEP_HEIGHT: u8 = 11;
pub const CREEP: ImageRaw<BinaryColor> =
    ImageRaw::new(include_bytes!("../../assets/RadiantCreep.raw"), 13);
pub const DIRE_CREEP: ImageRaw<BinaryColor> =
    ImageRaw::new(include_bytes!("../../assets/RadiantCreep.raw"), 13);

pub const SPLASH: ImageRaw<BinaryColor> =
    ImageRaw::new(include_bytes!("../../assets/Splash.raw"), 128);

#[derive(Debug)]
pub enum Sprite {
    Pudge,
    Creep(Creep),
    Hook,
    Cleaver,
}

impl Sprite {
    pub fn get_image(&self) -> ImageRaw<'static, BinaryColor> {
        match self {
            Sprite::Pudge => PUDGE_BODY,
            Sprite::Creep(creep_type) => match creep_type {
                Creep::Dire => CREEP,
                Creep::Radiant => CREEP,
            },
            Sprite::Hook => PUDGE_HOOK,
            Sprite::Cleaver => PUDGE_CLEAVER,
        }
    }
}
