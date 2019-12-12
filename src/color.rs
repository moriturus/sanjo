#![allow(dead_code)]

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct Color([u8; 4]);

impl From<&str> for Color {
    fn from(s: &str) -> Color {
        let s = s.chars().skip(1).collect::<String>();
        let code = if s.len() < 8 {
            u32::from_str_radix(&s, 16).unwrap() << 8 | 0x00_00_00_ff
        } else {
            u32::from_str_radix(&s, 16).unwrap()
        };
        Color::from_code(code)
    }
}

impl Into<image::Rgba<u8>> for Color {
    fn into(self) -> image::Rgba<u8> {
        image::Rgba(self.0)
    }
}

impl Color {
    pub fn from_code(code: u32) -> Color {
        let red = ((code & 0xff_00_00_00) >> 24) as u8;
        let green = ((code & 0x00_ff_00_00) >> 16) as u8;
        let blue = ((code & 0x00_00_ff_00) >> 8) as u8;
        let alpha = (code & 0x00_00_00_ff) as u8;
        Color::new(red, green, blue, alpha)
    }

    pub fn new(r: u8, g: u8, b: u8, a: u8) -> Color {
        Color([r, g, b, a])
    }

    pub fn clear() -> Color {
        Color([0, 0, 0, 0])
    }

    pub fn white() -> Color {
        Color([255, 255, 255, 255])
    }

    pub fn black() -> Color {
        Color([0, 0, 0, 255])
    }

    pub fn red() -> Color {
        Color([255, 0, 0, 255])
    }

    pub fn blue() -> Color {
        Color([0, 0, 255, 255])
    }

    pub fn green() -> Color {
        Color([0, 255, 0, 255])
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_from_code() {
        let code = 0xff_a5_00_ff;
        let color = Color::from_code(code);
        assert_eq!(&color.0, &[255, 165, 0, 255]);
    }

    #[test]
    fn test_from_str_rgba() {
        let code = "#ffa500ff";
        let color = Color::from(code);
        assert_eq!(&color.0, &[255, 165, 0, 255]);
    }

    #[test]
    fn test_from_str_rgb() {
        let code = "#ffa500";
        let color = Color::from(code);
        assert_eq!(&color.0, &[255, 165, 0, 255]);
    }

    #[test]
    fn test_clear() {
        let code = "#00000000";
        let color = Color::from(code);
        let clear = Color::clear();
        assert_eq!(color, clear);
    }

    #[test]
    fn test_white() {
        let code = "#ffffffff";
        let color = Color::from(code);
        let white = Color::white();
        assert_eq!(color, white);
    }

    #[test]
    fn test_black() {
        let code = "#000000ff";
        let color = Color::from(code);
        let black = Color::black();
        assert_eq!(color, black);
    }

    #[test]
    fn test_red() {
        let code = "#ff0000ff";
        let color = Color::from(code);
        let red = Color::red();
        assert_eq!(color, red);
    }

    #[test]
    fn test_green() {
        let code = "#00ff00ff";
        let color = Color::from(code);
        let green = Color::green();
        assert_eq!(color, green);
    }

    #[test]
    fn test_blue() {
        let code = "#0000ffff";
        let color = Color::from(code);
        let blue = Color::blue();
        assert_eq!(color, blue);
    }
}
