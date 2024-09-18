use core::panic;

use super::DisplayEnum;

pub fn flush(display: &mut DisplayEnum) {
    match display {
        DisplayEnum::WebView(ref mut disp) => disp.flush().unwrap(),
        _ => panic!("Trying to flush a non-flushable display!"),
    }
}

