use failure::{format_err, Error};
use std::str::Chars;

const MAX_COLOR_DIFF: f64 = 0.01;

#[derive(Copy, Clone, Debug)]
pub struct ColorRGBA {
    red: u8,
    green: u8,
    blue: u8,
    alpha: u8,
}

impl ColorRGBA {
    pub fn parse_string(color_string: &str) -> Result<Self, Error> {
        if color_string.len() != 7 {
            return Err(format_err!("Expected lenght of color string is 7, string lenght is {}", color_string.len()));
        }

        if !color_string.starts_with('#') {
            return Err(format_err!("Expected color string to start with '#'"));
        }

        let mut chars = color_string.chars();
        let _pound = chars.next(); // #

        let red = Self::decode_hex_color(&mut chars)?;
        let green = Self::decode_hex_color(&mut chars)?;
        let blue = Self::decode_hex_color(&mut chars)?;

        Ok(ColorRGBA {
            red: red,
            green: green,
            blue: blue,
            alpha: 255,
        })
    }

    pub fn red(self) -> u8 {
        self.red
    }

    pub fn red_normalized(self) -> f64 {
        f64::from(self.red) / 255.0
    }

    pub fn green(self) -> u8 {
        self.green
    }

    pub fn green_normalized(self) -> f64 {
        f64::from(self.green) / 255.0
    }

    pub fn blue(self) -> u8 {
        self.blue
    }

    pub fn blue_normalized(self) -> f64 {
        f64::from(self.blue) / 255.0
    }

    pub fn alpha(self) -> u8 {
        self.alpha
    }

    pub fn alpha_normalized(self) -> f64 {
        f64::from(self.alpha) / 255.0
    }

    pub fn adjust_lightness(&mut self, percentage: f64) -> Result<(), Error> {
        let mut hsla = self.to_hsla()?;
        hsla.lightness_percentage(percentage);
        let rgba = hsla.to_rgba()?;
        self.red = rgba.red();
        self.green = rgba.green();
        self.blue = rgba.blue();
        self.alpha = rgba.alpha();
        Ok(())
    }

    pub fn to_hsla(self) -> Result<ColorHSLA, Error> {
        let red_normalized = self.red_normalized();
        let green_normalized = self.green_normalized();
        let blue_normalized = self.blue_normalized();
        let c_max = Self::max_3(red_normalized, green_normalized, blue_normalized);
        let c_min = Self::min_3(red_normalized, green_normalized, blue_normalized);
        let delta = c_max - c_min;

        let hue;
        if delta.abs() < MAX_COLOR_DIFF {
            hue = 0.0;
        } else {
            if (c_max - red_normalized).abs() < MAX_COLOR_DIFF {
                hue = 60.0 * (((green_normalized - blue_normalized) / delta) % 6.0);
            } else if (c_max - green_normalized).abs() < MAX_COLOR_DIFF {
                hue = 60.0 * (((blue_normalized - red_normalized) / delta) + 2.0);
            } else if (c_max - blue_normalized).abs() < MAX_COLOR_DIFF {
                hue = 60.0 * (((red_normalized - green_normalized) / delta) + 4.0);
            } else {
                return Err(format_err!("c_max matches neither R, G or B"));
            }
        }

        let lightness = (c_max + c_min) / 2.0;
        let saturation = if delta != 0.0 { delta / (1.0 - ((2.0 * lightness) - 1.0).abs()) } else { 0.0 };

        Ok(ColorHSLA {
            hue: hue,
            saturation: saturation,
            lightness: lightness,
            alpha: 1.0,
        })
    }

    fn min_3(f_1: f64, f_2: f64, f_3: f64) -> f64 {
        Self::min_2(Self::min_2(f_1, f_2), f_3)
    }

    fn min_2(f_1: f64, f_2: f64) -> f64 {
        if f_1 <= f_2 {
            return f_1;
        }
        f_2
    }

    fn max_3(f_1: f64, f_2: f64, f_3: f64) -> f64 {
        Self::max_2(Self::max_2(f_1, f_2), f_3)
    }

    fn max_2(f_1: f64, f_2: f64) -> f64 {
        if f_1 >= f_2 {
            return f_1;
        }
        f_2
    }

    fn decode_hex_color(chars: &mut Chars) -> Result<u8, Error> {
        let c_1 = chars.next().ok_or(format_err!("some err"))?;
        let c_2 = chars.next().ok_or(format_err!("some err"))?;
        let c_1 = Self::decode_char(c_1)?;
        let c_2 = Self::decode_char(c_2)?;
        let color = c_1 * 16 + c_2;
        Ok(color)
    }

    fn decode_char(c: char) -> Result<u8, Error> {
        match c {
            '0' => Ok(0),
            '1' => Ok(1),
            '2' => Ok(2),
            '3' => Ok(3),
            '4' => Ok(4),
            '5' => Ok(5),
            '6' => Ok(6),
            '7' => Ok(7),
            '8' => Ok(8),
            '9' => Ok(9),
            'a' => Ok(10),
            'A' => Ok(10),
            'b' => Ok(11),
            'B' => Ok(11),
            'c' => Ok(12),
            'C' => Ok(12),
            'd' => Ok(13),
            'D' => Ok(13),
            'e' => Ok(14),
            'E' => Ok(14),
            'f' => Ok(15),
            'F' => Ok(15),
            _ => Err(format_err!("illegal character {}", c)),
        }
    }
}

#[derive(Copy, Clone, Debug)]
pub struct ColorHSLA {
    hue: f64,
    saturation: f64,
    lightness: f64,
    alpha: f64,
}

impl ColorHSLA {
    pub fn lightness_percentage(&mut self, percentage: f64) {
        let mut new_lightness = self.lightness * (1.0 + percentage);
        if new_lightness > 1.0 {
            new_lightness = 1.0;
        } else if new_lightness < 0.0 {
            new_lightness = 0.0;
        }
        self.lightness = new_lightness;
    }

    pub fn to_rgba(&self) -> Result<ColorRGBA, Error> {
        let c = (1.0 - ((2.0 * self.lightness) - 1.0).abs()) * self.saturation;
        let x = c * (1.0 - (((self.hue / 60.0) % 2.0) - 1.0).abs());
        let m = self.lightness - (c / 2.0);
        let mut r_n = 0.0;
        let mut g_n = 0.0;
        let mut b_n = 0.0;
        if self.hue >= 0.0 && self.hue < 60.0 {
            r_n = c;
            g_n = x;
            b_n = 0.0;
        } else if self.hue >= 60.0 && self.hue < 120.0 {
            r_n = x;
            g_n = c;
            b_n = 0.0;
        } else if self.hue >= 120.0 && self.hue < 180.0 {
            r_n = 0.0;
            g_n = c;
            b_n = x;
        } else if self.hue >= 180.0 && self.hue < 240.0 {
            r_n = 0.0;
            g_n = x;
            b_n = c;
        } else if self.hue >= 240.0 && self.hue < 300.0 {
            r_n = x;
            g_n = 0.0;
            b_n = c;
        } else if self.hue >= 300.0 && self.hue <= 360.0 {
            r_n = c;
            g_n = 0.0;
            b_n = x;
        } else if self.hue > 360.0 {
            return Err(format_err!("hue exceeds 360°"));
        }

        let red = ((r_n + m) * 255.0) as u8;
        let green = ((g_n + m) * 255.0) as u8;
        let blue = ((b_n + m) * 255.0) as u8;
        let alpha = (self.alpha * 255.0) as u8;

        Ok(ColorRGBA {
            red: red,
            green: green,
            blue: blue,
            alpha: alpha,
        })
    }
}

#[cfg(test)]
mod tests {
    use super::ColorHSLA;
    use super::ColorRGBA;

    #[test]
    fn parse_color_string() {
        let color_string = "#FF0077";
        let color = ColorRGBA::parse_string(&color_string).unwrap();
        assert_eq!(color.red, 255);
        assert_eq!(color.green, 0);
        assert_eq!(color.blue, 119);
        assert_eq!(color.alpha, 255);
    }

    #[test]
    fn rgba_to_hsla() {
        let rgba = ColorRGBA {
            red: 35,
            green: 200,
            blue: 200,
            alpha: 255,
        };
        let hsla = rgba.to_hsla().unwrap();
        assert_eq!(hsla.hue, 180.0);
        assert!(hsla.saturation > 0.69 && hsla.saturation < 0.71);
        assert!(hsla.lightness > 0.45 && hsla.lightness < 0.47);
        assert_eq!(hsla.alpha, 1.0);
    }

    #[test]
    fn hsla_to_rgba() {
        let hsla = ColorHSLA {
            hue: 180.0,
            saturation: 0.70,
            lightness: 0.46,
            alpha: 1.0,
        };
        let rgba = hsla.to_rgba().unwrap();
        assert_eq!(rgba.red, 35);
        assert_eq!(rgba.green, 199);
        assert_eq!(rgba.blue, 199);
        assert_eq!(rgba.alpha, 255);
    }
}
