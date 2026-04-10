use ratatui::style::Color;

pub struct OctopusTheme;

impl OctopusTheme {
    pub const PRIMARY: Color = Color::Rgb(138, 43, 226); // Purple
    pub const SECONDARY: Color = Color::Rgb(255, 127, 80); // Coral
    pub const ACCENT: Color = Color::Rgb(0, 206, 209); // Cyan
    pub const BACKGROUND: Color = Color::Rgb(30, 30, 46); // Dark blue-gray
    pub const FOREGROUND: Color = Color::Rgb(205, 214, 244); // Light text
    pub const SUCCESS: Color = Color::Rgb(166, 227, 161); // Green
    pub const WARNING: Color = Color::Rgb(249, 226, 175); // Yellow
    pub const ERROR: Color = Color::Rgb(243, 139, 168); // Red
}
