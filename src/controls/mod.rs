#[cfg(target_os = "none")]
use defmt::Format;

#[cfg(not(any(target_os = "none", target_os = "unknown")))]
pub mod native;

#[derive(PartialEq, Eq, Debug, Clone, Copy)]
pub enum ControlEnum {
    Left = -1,
    Right = 1,
    Hook,
    None,
}
impl ControlEnum {
    pub fn is_none(&self) -> bool {
        *self == ControlEnum::None
    }
    pub fn is_some(&self) -> bool {
        *self != ControlEnum::None
    }
}

#[cfg(target_os = "none")]
impl Format for ControlEnum {
    fn format(&self, f: defmt::Formatter) {
        match self {
            ControlEnum::Left => defmt::write!(f, "Left"),
            ControlEnum::Right => defmt::write!(f, "Right"),
            ControlEnum::Hook => defmt::write!(f, "Hook"),
            ControlEnum::None => defmt::write!(f, "None"),
        }
    }
}
