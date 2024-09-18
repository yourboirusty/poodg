use core::cell::RefCell;

use critical_section::Mutex;
use embedded_alloc::LlffHeap as Heap;
use embedded_graphics::geometry::Point;
use embedded_graphics::image::Image;
use embedded_hal::digital::InputPin;
use embedded_hal::digital::OutputPin;
use fugit::HertzU32;
use fugit::RateExtU32;
use pac::interrupt;

use defmt::*;
use rand::Rng;
use rp2040_hal::clocks::ClockSource;
use rp2040_hal::gpio;
use rp2040_hal::gpio::Interrupt;
use rp2040_hal::gpio::Pins;
use rp2040_hal::rosc::RingOscillator;
use rp2040_hal::Sio;
use rp2040_hal::Timer;
use rp2040_hal::{clocks::init_clocks_and_plls, pac, Watchdog, I2C};
extern crate cortex_m_rt;
extern crate ssd1309;
#[global_allocator]
static HEAP: Heap = Heap::empty();
use ssd1309::displayrotation::DisplayRotation;
use ssd1309::mode::GraphicsMode;
use ssd1309::Builder;

use crate::controls::ControlEnum;
use crate::game::Game;
use crate::graphics::display::clear_display;
use crate::graphics::display::DisplayEnum;
use crate::graphics::image::draw_image;
use crate::graphics::resources::SPLASH;

const XOSC_CRYSTAL_FREQ: HertzU32 = HertzU32::MHz(12);

enum Rotary {
    // 11 - default
    Rotary0,
    // 01 - starting clockwise
    Rotary1,
    // 00 - halfway
    Rotary2,
    // 10 - starting counter-clock
    Rotary3,
}

pub enum Direction {
    Clockwise,
    CounterClock,
}
type Enc1Pin = gpio::Pin<gpio::bank0::Gpio18, gpio::FunctionSioInput, gpio::PullUp>;
type Enc2Pin = gpio::Pin<gpio::bank0::Gpio19, gpio::FunctionSioInput, gpio::PullUp>;
type LEDPin = gpio::Pin<gpio::bank0::Gpio25, gpio::FunctionSioOutput, gpio::PullUp>;
type EncBtnPin = gpio::Pin<gpio::bank0::Gpio20, gpio::FunctionSioInput, gpio::PullUp>;
type EncoderPins = (Enc1Pin, Enc2Pin, EncBtnPin);

static ENCODER: Mutex<RefCell<Option<EncoderPins>>> = Mutex::new(RefCell::new(None));
static ACTION: Mutex<RefCell<[Option<ControlEnum>; 8]>> = Mutex::new(RefCell::new([None; 8]));

pub fn embed_main() -> ! {
    info!("Starting main");
    {
        use core::mem::MaybeUninit;
        const HEAP_SIZE: usize = 1024;
        static mut HEAP_MEM: [MaybeUninit<u8>; HEAP_SIZE] = [MaybeUninit::uninit(); HEAP_SIZE];
        unsafe { HEAP.init(HEAP_MEM.as_ptr() as usize, HEAP_SIZE) }
    }
    let mut pac = pac::Peripherals::take().unwrap();
    let core = pac::CorePeripherals::take().unwrap();
    let mut rosc = RingOscillator::new(pac.ROSC).initialize();
    let mut watchdog = Watchdog::new(pac.WATCHDOG);
    let clocks = init_clocks_and_plls(
        XOSC_CRYSTAL_FREQ.to_Hz(),
        pac.XOSC,
        pac.CLOCKS,
        pac.PLL_SYS,
        pac.PLL_USB,
        &mut pac.RESETS,
        &mut watchdog,
    )
    .ok()
    .unwrap();

    let sio = Sio::new(pac.SIO);
    let pins = Pins::new(
        pac.IO_BANK0,
        pac.PADS_BANK0,
        sio.gpio_bank0,
        &mut pac.RESETS,
    );
    // Configure display
    let mut disp_delay =
        cortex_m::delay::Delay::new(core.SYST, clocks.system_clock.get_freq().to_Hz());
    let mut reset = pins.gpio2.into_push_pull_output();
    let i2c = I2C::i2c0(
        pac.I2C0,
        pins.gpio4.reconfigure(), // sda
        pins.gpio5.reconfigure(), // scl
        400_u32.kHz(),
        &mut pac.RESETS,
        &clocks.peripheral_clock,
    );
    let i2c_interface = display_interface_i2c::I2CInterface::new(i2c, 0x3c, 0x40);
    let mut phys_disp: GraphicsMode<_> = Builder::new()
        .with_rotation(DisplayRotation::Rotate0)
        .connect(i2c_interface)
        .into();
    let _ = phys_disp.reset(&mut reset, &mut disp_delay);
    phys_disp.init().unwrap();
    let mut display = DisplayEnum::Oled(phys_disp);
    let splash = Image::new(&SPLASH, Point::zero());
    draw_image(&mut display, splash);
    if let DisplayEnum::Oled(ref mut disp) = display {
        disp.flush().unwrap()
    }
    info!("Flushed!");
    let mut game = Game::default();
    let timer = Timer::new(pac.TIMER, &mut pac.RESETS, &clocks);

    let enc_1: Enc1Pin = pins.gpio18.reconfigure();
    let enc_2: Enc2Pin = pins.gpio19.reconfigure();
    let enc_btn: EncBtnPin = pins.gpio20.reconfigure();
    let mut led: LEDPin = pins.gpio25.reconfigure();
    let _ = led.set_high();
    enc_1.set_interrupt_enabled(gpio::Interrupt::EdgeHigh, true);
    enc_2.set_interrupt_enabled(gpio::Interrupt::EdgeHigh, true);
    enc_1.set_interrupt_enabled(gpio::Interrupt::EdgeLow, true);
    enc_2.set_interrupt_enabled(gpio::Interrupt::EdgeLow, true);
    enc_btn.set_interrupt_enabled(Interrupt::EdgeLow, true);
    critical_section::with(|cs| {
        ENCODER.borrow(cs).replace(Some((enc_1, enc_2, enc_btn)));
    });
    unsafe {
        pac::NVIC::unmask(pac::Interrupt::IO_IRQ_BANK0);
    }
    game.init();
    game.set_seed(rosc.gen());
    loop {
        main_loop(&mut display, &mut game, &timer);
    }
}

pub fn main_loop(display: &mut DisplayEnum, game: &mut Game, timer: &Timer) {
    let tick = timer.get_counter();
    let mut action_queue: [Option<ControlEnum>; 8] = [None; 8];
    critical_section::with(|cs| {
        action_queue = (*ACTION.borrow(cs)).take();
    });

    for action in action_queue {
        if action.is_none() {
            break;
        }
        game.control(action.unwrap());
    }

    game.process(tick);

    clear_display(display);
    game.draw(display);
    if let DisplayEnum::Oled(ref mut disp) = display {
        disp.flush().unwrap()
    }
}

#[interrupt]
fn IO_IRQ_BANK0() {
    static mut ENCODER_PINS: Option<EncoderPins> = None;
    static mut ROTARY: Rotary = Rotary::Rotary0;
    static mut DIRECTION: Option<Direction> = None;

    debug!("Interrupt!");
    if ENCODER_PINS.is_none() {
        critical_section::with(|cs| {
            *ENCODER_PINS = ENCODER.borrow(cs).take();
        });
    }

    if let Some(gpios) = ENCODER_PINS {
        let (enc_a, enc_b, enc_btn) = gpios;
        enc_a.clear_interrupt(gpio::Interrupt::EdgeLow);
        enc_a.clear_interrupt(gpio::Interrupt::EdgeHigh);
        enc_b.clear_interrupt(gpio::Interrupt::EdgeLow);
        enc_b.clear_interrupt(gpio::Interrupt::EdgeHigh);
        enc_btn.clear_interrupt(Interrupt::EdgeLow);
        let mut new_action: Option<ControlEnum> = None;

        if let (Ok(enc1_hi), Ok(enc2_hi)) = (enc_a.is_high(), enc_b.is_high()) {
            match (&ROTARY, &DIRECTION, enc1_hi, enc2_hi) {
                // Rotate clockwise
                (Rotary::Rotary0, _, false, true) => {
                    *DIRECTION = Some(Direction::Clockwise);
                    *ROTARY = Rotary::Rotary1;
                }
                (Rotary::Rotary3, Some(Direction::Clockwise), true, true) => {
                    new_action = Some(ControlEnum::Right);
                    *ROTARY = Rotary::Rotary0;
                    *DIRECTION = None;
                }

                // Rotate counter-clock
                (Rotary::Rotary0, _, true, false) => {
                    *DIRECTION = Some(Direction::CounterClock);
                    *ROTARY = Rotary::Rotary3;
                }
                (Rotary::Rotary1, Some(Direction::CounterClock), true, true) => {
                    new_action = Some(ControlEnum::Left);
                    *ROTARY = Rotary::Rotary0;
                    *DIRECTION = None;
                }

                // misc
                (_, _, true, true) => {
                    *ROTARY = Rotary::Rotary0;
                    *DIRECTION = None;
                }
                (_, _, false, true) => {
                    *ROTARY = Rotary::Rotary1;
                }
                (_, _, false, false) => {
                    *ROTARY = Rotary::Rotary2;
                }
                (_, _, true, false) => {
                    *ROTARY = Rotary::Rotary3;
                }
            }
            critical_section::with(|cs| {
                if new_action.is_some() {
                    let mut actions = ACTION.borrow(cs).take();
                    for action in actions.iter_mut() {
                        if action.is_none() {
                            *action = Some(new_action.unwrap());
                            break;
                        }
                    }
                    ACTION.replace(cs, actions);
                }
            });
        }

        if let Ok(enc_btn_hi) = enc_btn.is_high() {
            if enc_btn_hi {
                return;
            }
            debug!("Hooking!");
            critical_section::with(|cs| {
                let mut actions = ACTION.borrow(cs).take();
                for action in actions.iter_mut() {
                    if action.is_none() {
                        *action = Some(ControlEnum::Hook);
                        break;
                    }
                }
                ACTION.replace(cs, actions);
            });
        }
    }
}
