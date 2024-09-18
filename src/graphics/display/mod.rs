#[cfg(not(any(target_os = "none", target_os = "unknown")))]
pub mod simulator;

#[cfg(target_arch = "wasm32")]
pub mod webview;

use embedded_graphics::prelude::*;
use embedded_graphics::{mock_display::MockDisplay, pixelcolor::BinaryColor};

#[cfg(not(any(target_os = "none", target_os = "unknown")))]
use embedded_graphics_simulator::SimulatorDisplay;
#[cfg(target_arch = "wasm32")]
use embedded_graphics_web_simulator::display::WebSimulatorDisplay;
#[cfg(target_os = "none")]
use libm::floorf;
#[cfg(target_os = "none")]
use rp2040_hal::{pac, I2C};
#[cfg(target_os = "none")]
use ssd1309::mode::GraphicsMode;

pub const DISPLAY_WIDTH: u32 = 128;
pub const DISPLAY_HEIGHT: u32 = 64;

#[cfg(target_os = "none")]
type OledDisplay = GraphicsMode<
    display_interface_i2c::I2CInterface<
        I2C<
            pac::I2C0,
            (
                rp2040_hal::gpio::Pin<
                    rp2040_hal::gpio::bank0::Gpio4,
                    rp2040_hal::gpio::FunctionI2c,
                    rp2040_hal::gpio::PullUp,
                >,
                rp2040_hal::gpio::Pin<
                    rp2040_hal::gpio::bank0::Gpio5,
                    rp2040_hal::gpio::FunctionI2c,
                    rp2040_hal::gpio::PullUp,
                >,
            ),
        >,
    >,
>;

pub enum DisplayEnum {
    #[cfg(not(any(target_os = "none", target_os = "unknown")))]
    Simulator(SimulatorDisplay<BinaryColor>),
    #[cfg(target_arch = "wasm32")]
    WebView(WebSimulatorDisplay<BinaryColor>),
    #[cfg(target_os = "none")]
    Oled(OledDisplay),
    Mock(MockDisplay<BinaryColor>),
}

#[cfg(not(any(target_os = "none", target_os = "unknown")))]
pub fn get_display() -> DisplayEnum {
    let display = if cfg!(not(any(target_os = "none", target_os = "unknown"))) {
        DisplayEnum::Simulator(SimulatorDisplay::<BinaryColor>::new(Size::new(128, 64)))
    } else {
        DisplayEnum::Mock(MockDisplay::new())
    };
    display
}

#[cfg(target_arch = "wasm32")]
pub fn get_display() -> DisplayEnum {
    use embedded_graphics_web_simulator::output_settings::OutputSettingsBuilder;

    let settings = OutputSettingsBuilder::new()
        .scale(3)
        .pixel_spacing(1)
        .build();

    let display = if cfg!(not(target_os = "none")) {
        DisplayEnum::WebView(WebSimulatorDisplay::<BinaryColor>::new(
            (DISPLAY_WIDTH, DISPLAY_HEIGHT),
            &settings,
            None,
        ))
    } else {
        DisplayEnum::Mock(MockDisplay::new())
    };
    display
}

pub fn clear_display(display: &mut DisplayEnum) {
    match *display {
        DisplayEnum::Mock(ref mut disp) => disp.clear(BinaryColor::Off).unwrap(),
        #[cfg(not(any(target_os = "none", target_os = "unknown")))]
        DisplayEnum::Simulator(ref mut disp) => disp.clear(BinaryColor::Off).unwrap(),
        #[cfg(target_arch = "wasm32")]
        DisplayEnum::WebView(ref mut disp) => disp.clear(BinaryColor::Off).unwrap(),
        #[cfg(target_arch = "arm")]
        DisplayEnum::Oled(ref mut disp) => disp.clear(),
    }
}

pub fn get_fps(delta: fugit::Duration<u64, 1, 1000000>) -> u32 {
    let fps = 1.0 / (delta.to_millis() as f32 / 1_000.0);
    #[cfg(not(target_os = "none"))]
    return fps.floor() as u32;
    #[cfg(target_os = "none")]
    return floorf(fps) as u32;
}
