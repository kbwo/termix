//! Defines all colors and UI styling.

use std::collections::HashMap;
use std::str::FromStr;

// ref: https://man7.org/linux/man-pages/man4/console_codes.4.html
#[derive(Debug, Clone)]
pub enum Color {
    Inherit,
    Ansi16(Ansi16Value),
    Ansi256(u32),
    Rgb(u8, u8, u8),
}

impl FromStr for Color {
    type Err = std::num::ParseIntError;

    // Parses a color hex code of the form '#rRgGbB..' into an
    // instance of 'RGB'
    fn from_str(hex_code: &str) -> Result<Self, Self::Err> {
        // u8::from_str_radix(src: &str, radix: u32) converts a string
        // slice in a given base to u8
        let r: u8 = u8::from_str_radix(&hex_code[1..3], 16)?;
        let g: u8 = u8::from_str_radix(&hex_code[3..5], 16)?;
        let b: u8 = u8::from_str_radix(&hex_code[5..7], 16)?;

        Ok(Color::Rgb(r, g, b))
    }
}

/// Utility struct for styled text
pub struct StyledText {
    text: String,
    fg: Color,
    bg: Color,
    bold: bool,
    underline: bool,
    reverse: bool,
}

#[derive(Debug, Clone, Eq, PartialEq, Hash, Copy)]
pub enum Ansi16Value {
    Black,
    Red,
    Green,
    Yellow,
    Blue,
    Magenta,
    Cyan,
    White,
    LightBlack,
    LightRed,
    LightGreen,
    LightYellow,
    LightBlue,
    LightMagenta,
    LightCyan,
    LightWhite,
}

impl Color {}

impl Default for Color {
    fn default() -> Self {
        Color::Inherit
    }
}

impl Default for StyledText {
    fn default() -> Self {
        StyledText {
            text: String::new(),
            fg: Color::Inherit,
            bg: Color::Inherit,
            bold: false,
            underline: false,
            reverse: false,
        }
    }
}

impl StyledText {
    const RESET: &str = "\x1b[0m";
    const BOLD: &str = "\x1b[1m";
    const UNDERLINE: &str = "\x1b[4m";
    const REVERSE: &str = "\x1b[7m";

    pub fn new(
        text: &str,
        fg: Option<Color>,
        bg: Option<Color>,
        bold: Option<bool>,
        underline: Option<bool>,
        reverse: Option<bool>,
    ) -> StyledText {
        let fg = fg.unwrap_or(Color::Inherit);
        let bg = bg.unwrap_or(Color::Inherit);
        let bold = bold.unwrap_or(false);
        let underline = underline.unwrap_or(false);
        let reverse = reverse.unwrap_or(false);
        StyledText {
            text: text.to_string(),
            fg,
            bg,
            bold,
            underline,
            reverse,
        }
    }

    pub fn text(&self) -> String {
        self.build_text()
    }

    fn build_text(&self) -> String {
        let mut styled_text = String::new();
        if self.bold {
            styled_text += StyledText::BOLD;
        }
        if self.underline {
            styled_text += StyledText::UNDERLINE;
        }
        if self.reverse {
            styled_text += StyledText::REVERSE;
        }
        styled_text + &self.fg_text() + &self.bg_text() + &self.text + StyledText::RESET
    }

    fn fg_text(&self) -> String {
        fg_color(&self.fg)
    }

    fn bg_text(&self) -> String {
        bg_color(&self.bg)
    }
}

fn fg_color(fg: &Color) -> String {
    let fg_ansi: HashMap<Ansi16Value, &str> = HashMap::from([
        (Ansi16Value::Black, "\x1b[30m"),
        (Ansi16Value::Red, "\x1b[31m"),
        (Ansi16Value::Green, "\x1b[32m"),
        (Ansi16Value::Yellow, "\x1b[33m"),
        (Ansi16Value::Blue, "\x1b[34m"),
        (Ansi16Value::Magenta, "\x1b[35m"),
        (Ansi16Value::Cyan, "\x1b[36m"),
        (Ansi16Value::White, "\x1b[37m"),
        (Ansi16Value::LightBlack, "\x1b[90m"),
        (Ansi16Value::LightRed, "\x1b[91m"),
        (Ansi16Value::LightGreen, "\x1b[92m"),
        (Ansi16Value::LightYellow, "\x1b[93m"),
        (Ansi16Value::LightBlue, "\x1b[94m"),
        (Ansi16Value::LightMagenta, "\x1b[95m"),
        (Ansi16Value::LightCyan, "\x1b[96m"),
        (Ansi16Value::LightWhite, "\x1b[97m"),
    ]);
    match fg {
        Color::Inherit => String::from("\x1b[39m"),
        Color::Ansi16(ansi) => fg_ansi.get(ansi).unwrap().to_string(),
        Color::Ansi256(x) => format!("\x1b[38;5;{}m", x),
        Color::Rgb(r, g, b) => format!("\x1b[38;2;{};{};{}m", r, g, b),
    }
}
fn bg_color(bg: &Color) -> String {
    let bg_ansi: HashMap<Ansi16Value, &str> = HashMap::from([
        (Ansi16Value::Black, "\x1b[40m"),
        (Ansi16Value::Red, "\x1b[41m"),
        (Ansi16Value::Green, "\x1b[42m"),
        (Ansi16Value::Yellow, "\x1b[43m"),
        (Ansi16Value::Blue, "\x1b[44m"),
        (Ansi16Value::Magenta, "\x1b[45m"),
        (Ansi16Value::Cyan, "\x1b[46m"),
        (Ansi16Value::White, "\x1b[47m"),
        (Ansi16Value::LightBlack, "\x1b[100m"),
        (Ansi16Value::LightRed, "\x1b[101m"),
        (Ansi16Value::LightGreen, "\x1b[102m"),
        (Ansi16Value::LightYellow, "\x1b[103m"),
        (Ansi16Value::LightBlue, "\x1b[104m"),
        (Ansi16Value::LightMagenta, "\x1b[105m"),
        (Ansi16Value::LightCyan, "\x1b[106m"),
        (Ansi16Value::LightWhite, "\x1b[107m"),
    ]);
    match bg {
        Color::Inherit => String::from("\x1b[49m"),
        Color::Ansi16(ansi) => bg_ansi.get(ansi).unwrap().to_string(),
        Color::Ansi256(x) => format!("\x1b[48;5;{}m", x),
        Color::Rgb(r, g, b) => format!("\x1b[48;2;{};{};{}m", r, g, b),
    }
}

#[cfg(test)]
mod tests {
    use std::str::FromStr;

    use crate::color::{bg_color, fg_color};

    use super::{Color, StyledText};

    #[test]
    fn hex_to_color() {
        vec![
            "#b14ffe", "#af53fd", "#ae58fb", "#ac5cfa", "#ab60f8", "#a964f7", "#a868f6", "#a76cf4",
            "#a66ff3", "#a472f2", "#a375f1", "#a279ef", "#a07cee", "#9f7eed", "#9e81ec", "#9d84eb",
            "#9b87ea", "#9a8ae9", "#998ce7", "#988fe6", "#9791e5", "#9594e4", "#9496e3", "#9399e2",
            "#929be1", "#909ee0", "#8fa0df", "#8ea2de", "#8ca5dd", "#8ba7dc", "#8aa9db", "#88acda",
            "#87aed9", "#85b0d7", "#84b2d6", "#83b4d5", "#81b7d4", "#80b9d3", "#7ebbd2", "#7cbdd1",
            "#7bbfd0", "#79c1cf", "#77c4cd", "#76c6cc", "#74c8cb", "#72caca", "#70ccc9", "#6ecec7",
            "#6cd0c6", "#6ad2c5", "#68d4c4", "#66d6c2", "#64d8c1", "#61dac0", "#5fdcbe", "#5cdebd",
            "#5ae0bc", "#57e2ba", "#54e5b9", "#51e7b7", "#4ee9b6", "#4aebb4", "#47edb2", "#43efb1",
            "#3ff1af", "#3af3ae", "#35f5ac", "#2ff7aa", "#28f9a8", "#20fba6", "#14fda4",
        ]
        .iter()
        .for_each(|hex| assert!(Color::from_str(hex).is_ok()));
        assert!(Color::from_str("testooooo").is_err());
    }

    #[test]
    fn non_hex_to_color() {
        assert!(Color::from_str("testooooo").is_err());
        assert!(Color::from_str("#3af3az").is_err());
    }

    #[test]
    fn build_styles() {
        let base = String::from("Hello, world");
        let styled_text = StyledText::new(
            &base,
            Some(Color::Ansi16(super::Ansi16Value::Red)),
            Some(Color::Rgb(4, 6, 8)),
            Some(true),
            Some(true),
            Some(true),
        );
        let res = styled_text.text();
        assert!(res.contains(&fg_color(&Color::Ansi16(super::Ansi16Value::Red))));
        assert!(res.contains(&bg_color(&Color::Rgb(4, 6, 8))));
        assert!(res.contains(StyledText::BOLD));
        assert!(res.contains(StyledText::UNDERLINE));
        assert!(res.contains(StyledText::REVERSE));
    }
}
