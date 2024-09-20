#![cfg_attr(target_os = "none", no_std)]
#![cfg_attr(target_os = "none", no_main)]

pub mod controls;
pub mod game;
pub mod graphics;

#[cfg(target_os = "none")]
mod embed;
#[cfg(not(any(target_os = "none", target_os = "unknown")))]
mod native;
#[cfg(target_arch = "wasm32")]
mod wasm;

#[cfg(target_os = "none")]
use bsp::entry;
#[cfg(target_os = "none")]
use rp_pico as bsp;

use embedded_graphics::image::Image;
use embedded_graphics::pixelcolor::BinaryColor;

type SpriteImage = Image<'static, embedded_graphics::image::ImageRaw<'static, BinaryColor>>;

#[cfg(not(any(target_os = "none", target_os = "unknown")))]
fn main() -> Result<(), core::convert::Infallible> {
    use native::native_main;
    native_main()
}

#[cfg(target_arch = "wasm32")]
fn main() {
    use wasm::wasm_main;

    wasm_main()
}

#[cfg(target_os = "none")]
#[entry]
fn main() -> ! {
    use defmt::*;
    use defmt_rtt as _;
    use embed::embed_main;
    use panic_probe as _;

    embed_main()
}
