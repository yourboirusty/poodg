use embedded_graphics::prelude::*;
use embedded_graphics::text::renderer::TextRenderer;
use embedded_graphics::text::Text;
use embedded_graphics::{
    image::{Image, ImageDrawable},
    pixelcolor::BinaryColor,
};

use super::display::{DisplayEnum, DISPLAY_WIDTH};
use super::resources::TEXT_STYLE;

pub fn draw_text<'a, T>(display: &mut DisplayEnum, text: Text<'a, T>)
where
    T: TextRenderer<Color = BinaryColor>,
{
    match display {
        #[cfg(target_os = "none")]
        DisplayEnum::Oled(ref mut disp) => {
            text.draw(disp).unwrap();
        }
        #[cfg(target_arch = "wasm32")]
        DisplayEnum::WebView(ref mut disp) => {
            text.draw(disp).unwrap();
        }
        _ => todo!(),
    }
}

pub fn draw_hp(display: &mut DisplayEnum, hp: u8) {
    let hp_str = "<3 ".repeat(hp.into());
    let hp_text = Text::with_alignment(
        &hp_str,
        Point::new(DISPLAY_WIDTH as i32 - 4, 6),
        TEXT_STYLE.clone(),
        embedded_graphics::text::Alignment::Right,
    );
    match display {
        #[cfg(target_os = "none")]
        DisplayEnum::Oled(ref mut disp) => {
            hp_text.draw(disp).unwrap();
        }
        #[cfg(target_arch = "wasm32")]
        DisplayEnum::WebView(ref mut disp) => {
            hp_text.draw(disp).unwrap();
        }
        _ => {
            todo!()
        }
    }
}

pub fn draw_image<T>(display: &mut DisplayEnum, image: Image<'_, T>)
where
    T: ImageDrawable<Color = BinaryColor>,
{
    match display {
        DisplayEnum::Mock(ref mut disp) => image.draw(disp).unwrap(),
        #[cfg(not(any(target_os = "none", target_os = "unknown")))]
        DisplayEnum::Simulator(ref mut disp) => image.draw(disp).unwrap(),
        #[cfg(target_arch = "wasm32")]
        DisplayEnum::WebView(ref mut disp) => image.draw(disp).unwrap(),
        #[cfg(target_os = "none")]
        DisplayEnum::Oled(ref mut phys_disp) => image.draw(phys_disp).unwrap(),
    };
}
