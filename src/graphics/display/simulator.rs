use embedded_graphics_simulator::{BinaryColorTheme, OutputSettingsBuilder, Window};

pub fn create_window() -> Window {
    let output_settings = OutputSettingsBuilder::new()
        .theme(BinaryColorTheme::OledBlue)
        .build();
    let window = Window::new("Poodg", &output_settings);

    window
}
