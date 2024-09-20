use core::{
    cell::{OnceCell, RefCell},
    panic,
};
use std::{
    rc::Rc,
    sync::{Arc, Mutex},
    time::Instant,
};

use embedded_graphics::{image::Image, prelude::Point};
use once_cell::sync::Lazy;
use wasm_bindgen::prelude::*;
use wasm_timer::SystemTime;
use web_sys::{console::log_1, MouseEvent, WheelEvent};

#[wasm_bindgen]
extern "C" {
    // Use `js_namespace` here to bind `console.log(..)` instead of just
    // `log(..)`
    #[wasm_bindgen(js_namespace = console)]
    fn log(s: &str);

    // The `console.log` is quite polymorphic, so we can bind it with multiple
    // signatures. Note that we need to use `js_name` to ensure we always call
    // `log` in JS.
    #[wasm_bindgen(js_namespace = console, js_name = log)]
    fn log_u32(a: u32);

    // Multiple arguments too!
    #[wasm_bindgen(js_namespace = console, js_name = log)]
    fn log_many(a: &str, b: &str);
}

macro_rules! console_log {
    // Note that this is using the `log` function imported above during
    // `bare_bones`
    ($($t:tt)*) => (log(&format_args!($($t)*).to_string()))
}

use crate::{
    controls::ControlEnum,
    game::Game,
    graphics::{
        display::{clear_display, get_display, webview::flush},
        image::draw_image,
        resources::SPLASH,
    },
};

use crate::game::Instant as GameInstant;

const NUM_ITER: i32 = 1;

enum ScrollDir {
    Up,
    Down,
}

struct WebMouse {
    scroll: Option<ScrollDir>,
    click: bool,
}

fn window() -> web_sys::Window {
    web_sys::window().expect("no global `window` exists")
}
fn perf() -> web_sys::Performance {
    window().performance().expect("no perfromance")
}
fn now() -> f64 {
    perf().now()
}

fn timeouted_frame(f: &Closure<dyn FnMut()>) {
    window()
        .request_animation_frame(f.as_ref().unchecked_ref())
        .expect("should register `requestAnimationFrame` OK");
}

fn request_animation_frame(f: &Closure<dyn FnMut()>) {
    window()
        .request_animation_frame(f.as_ref().unchecked_ref())
        .expect("should register `requestAnimationFrame` OK");
}

fn document() -> web_sys::Document {
    window()
        .document()
        .expect("should have a document on window")
}

static mut MOUSE: Option<Arc<Mutex<WebMouse>>> = None;
static START_TIME: Lazy<Arc<Mutex<OnceCell<SystemTime>>>> =
    Lazy::new(|| -> _ { return Arc::new(Mutex::new(OnceCell::new())) });

const LOGIC_TIMEOUT: i32 = 20;
pub fn wasm_main() {
    #[cfg(debug_assertions)]
    console_error_panic_hook::set_once();
    println!("Starting wasm");

    unsafe {
        MOUSE = Some(Arc::new(Mutex::new(WebMouse {
            scroll: None,
            click: false,
        })));
    }

    let document = document();
    let body = document.body().expect("document should have a body");

    let mut display = get_display();
    let splash = Image::new(&SPLASH, Point::zero());
    //console_log!("Splashin");
    draw_image(&mut display, splash);
    flush(&mut display);

    //console_log!("Clockin");
    #[cfg(not(target_os = "none"))]
    {
        let time_ref = START_TIME.clone();
        time_ref
            .lock()
            .expect("Couldn't lock time for setting")
            .set(SystemTime::now())
            .unwrap();
    }

    //console_log!("Registering clicks");
    // Set up click event listener
    let click_closure = {
        let mouse_click_ref = unsafe { MOUSE.clone().unwrap() };
        Closure::wrap(Box::new(move |event: MouseEvent| {
            // Only count left-click (button 0)
            if event.button() == 0 {
                let mut mouse = mouse_click_ref.lock().unwrap();
                mouse.click = true;
            }
        }) as Box<dyn FnMut(_)>)
    };

    body.add_event_listener_with_callback("click", click_closure.as_ref().unchecked_ref())
        .expect("Failed to add event listener");
    click_closure.forget();

    let scroll_closure = {
        let mouse_scroll_ref = unsafe { MOUSE.clone().unwrap() };
        Closure::wrap(Box::new(move |event: WheelEvent| {
            let mut mouse = mouse_scroll_ref.lock().unwrap();
            let delta = event.delta_y();
            if delta > 0.0 {
                mouse.scroll = Some(ScrollDir::Up);
            }
            if delta < 0.0 {
                mouse.scroll = Some(ScrollDir::Down);
            }
        }) as Box<dyn FnMut(_)>)
    };

    body.add_event_listener_with_callback("wheel", scroll_closure.as_ref().unchecked_ref())
        .expect("Failed to add event listener");
    scroll_closure.forget();

    let graphics_anchor: Rc<RefCell<Option<Closure<dyn FnMut()>>>> = Rc::new(RefCell::new(None));
    let graphics_ref = graphics_anchor.clone();

    let mouse_window_ref = unsafe { MOUSE.clone().unwrap() };
    let mut game = Game::default();
    game.init();

    *graphics_ref.borrow_mut() = Some(Closure::wrap(Box::new(move || {
        let ticks: u64 = (now() as u64) * 1000;
        let game_instant = GameInstant::from_ticks(ticks);

        // Set the body's text content to how many times this
        // requestAnimationFrame callback has fired.

        let mut control = ControlEnum::None;
        {
            let mut mouse = mouse_window_ref.lock().unwrap();
            if mouse.scroll.is_some() {
                let scroll = mouse.scroll.as_ref().unwrap();
                match scroll {
                    ScrollDir::Up => control = ControlEnum::Right,
                    ScrollDir::Down => control = ControlEnum::Left,
                }
                mouse.scroll = None;
            }
            if mouse.click {
                control = ControlEnum::Hook
            }
            mouse.click = false;
        }

        clear_display(&mut display);

        game.draw(&mut display);

        flush(&mut display);
        game.process(game_instant);
        game.control(control);

        // Schedule ourself for another requestAnimationFrame callback.
        set_timeout(graphics_anchor.borrow().as_ref().unwrap(), LOGIC_TIMEOUT);
    }) as Box<dyn FnMut()>));

    set_timeout(graphics_ref.borrow().as_ref().unwrap(), LOGIC_TIMEOUT);
}
fn set_timeout(f: &Closure<dyn FnMut()>, timeout_ms: i32) -> i32 {
    window()
        .set_timeout_with_callback_and_timeout_and_arguments_0(
            f.as_ref().unchecked_ref(),
            timeout_ms,
        )
        .expect("should register `setTimeout` OK")
}
