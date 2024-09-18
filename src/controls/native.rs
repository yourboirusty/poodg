use embedded_graphics::prelude::Point;
use embedded_graphics_simulator::{
    sdl2::{Keycode, MouseButton, MouseWheelDirection},
    SimulatorEvent,
};

use super::ControlEnum;

pub struct WheelControl {
    pub wheel_delta: Point,
    pub direction: MouseWheelDirection,
}

impl From<WheelControl> for ControlEnum {
    fn from(value: WheelControl) -> Self {
        if value.wheel_delta.y > 0 {
            return ControlEnum::Left;
        }

        if value.wheel_delta.y < 0 {
            return ControlEnum::Right;
        }
        ControlEnum::None
    }
}

impl From<MouseButton> for ControlEnum {
    fn from(value: MouseButton) -> Self {
        if value == MouseButton::Left {
            return ControlEnum::Hook;
        }
        ControlEnum::None
    }
}

impl From<Keycode> for ControlEnum {
    fn from(value: Keycode) -> Self {
        match value {
            Keycode::Left | Keycode::A => ControlEnum::Left,
            Keycode::Right | Keycode::D => ControlEnum::Right,
            Keycode::Space | Keycode::W => ControlEnum::Hook,
            _ => ControlEnum::None,
        }
    }
}

pub fn window_controls(
    win: &mut embedded_graphics_simulator::Window,
    controls: &mut Option<ControlEnum>,
) -> Result<(), ()> {
    Ok(for event in win.events() {
        // Will be easier to refactor after controller interface implementation
        match event {
            SimulatorEvent::Quit => Err(()),
            SimulatorEvent::MouseWheel {
                scroll_delta,
                direction,
            } => {
                let wheel_control = ControlEnum::from(WheelControl {
                    wheel_delta: scroll_delta,
                    direction,
                });
                if wheel_control.is_some() {
                    *controls = Some(wheel_control);
                }
                Ok(())
            }
            SimulatorEvent::KeyDown {
                keycode,
                keymod,
                repeat,
            } => {
                let kbd_control = ControlEnum::from(keycode);
                if kbd_control.is_some() {
                    *controls = Some(kbd_control);
                }
                Ok(())
            }
            SimulatorEvent::MouseButtonDown { mouse_btn, point } => {
                let mouse_control = ControlEnum::from(mouse_btn);
                if mouse_control.is_some() {
                    *controls = Some(mouse_control);
                }
                Ok(())
            }
            _ => Ok(()),
        }?;
    })
}
