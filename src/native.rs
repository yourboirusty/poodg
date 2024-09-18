use graphics::display::clear_display;

use game::Instant;

use crate::controls::native::window_controls;
use crate::{
    controls::ControlEnum,
    graphics::{display, resources::SPLASH},
};

use core::time::Duration;

use std::thread::sleep;

use graphics::display::DisplayEnum;

use game::Game;

use crate::{game, graphics};

use graphics::image::draw_image;

use embedded_graphics::{image::Image, prelude::*};

use graphics::display::get_display;
use once_cell::sync::OnceCell;
use std::time::SystemTime;

static START_TIME: OnceCell<SystemTime> = OnceCell::new();

pub(crate) fn native_main() -> Result<(), core::convert::Infallible> {
    let mut window = if cfg!(not(target_os = "none")) {
        println!("Creating window");
        Some(display::simulator::create_window())
    } else {
        None
    };

    println!("Display init");
    let mut display = get_display();

    let splash = Image::new(&SPLASH, Point::new(0, 0));
    draw_image(&mut display, splash);

    println!("Game init");
    let mut game = Game::default();
    game.init();

    #[cfg(not(target_os = "none"))]
    if let DisplayEnum::Simulator(ref mut disp) = display {
        if let Some(ref mut win) = window {
            win.update(disp)
        }
    }

    println!("Time init");
    #[cfg(not(target_os = "none"))]
    START_TIME.set(SystemTime::now()).unwrap();

    sleep(Duration::from_secs(1));
    'running: loop {
        let err = main_loop(&mut window, &mut display, &mut game);
        if err.is_err() {
            break 'running;
        }
    }
    Ok(())
}

pub(crate) fn main_loop(
    window: &mut Option<embedded_graphics_simulator::Window>,
    display: &mut DisplayEnum,
    game: &mut Game,
) -> Result<(), ()> {
    let mut controls: Option<ControlEnum> = None;

    let ticks: u64 = if cfg!(not(target_os = "none")) {
        let delta = START_TIME.get().unwrap().elapsed().expect("NO TIIIIME");
        delta.as_micros().try_into().unwrap()
    } else {
        panic!("NO TIME")
    };
    let clock = Instant::from_ticks(ticks);

    if let Some(ref mut win) = *window {
        clear_display(display);
        if let DisplayEnum::Simulator(ref _displ) = *display {
            window_controls(win, &mut controls)?;
        }

        game.draw(display);

        if let DisplayEnum::Simulator(ref displ) = *display {
            win.update(displ);
        }
    }

    if let Some(ctrl) = controls {
        game.control(ctrl)
    }
    game.process(clock);
    Ok(())
}
