use std::str::Chars;
use failure::{
    Error,
    format_err,
};

pub struct ColorRGBA {
    pub red: f32,
    pub green: f32,
    pub blue: f32,
    pub alpha: f32,
}

impl ColorRGBA {
    pub fn parse_string(color_string: &str) -> Result<Self, Error> {
        if color_string.len() != 7 {
            return Err(format_err!("Expected lenght of color string is 7, string lenght is {}", color_string.len()))
        }

        if !color_string.starts_with("#") {
            return Err(format_err!("Expected color string to start with '#'"))
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
            alpha: 1.0,
        })
    }

    pub fn to_hsla(&self) -> Result<ColorHSLA, Error> {
        let c_max = Self::max_3(self.red, self.green, self.blue);
        let c_min = Self::min_3(self.red, self.green, self.blue);
        let delta = c_max - c_min;


        let mut hue = 0.0;
        if delta == 0.0 {
            hue = 0.0;
        }
        else {
            if c_max == self.red {
                hue = 60.0 * (((self.green - self.blue) / delta) % 6.0);
            }
            else if c_max == self.green {
                hue = 60.0 * (((self.blue - self.red) / delta) + 2.0);
            }
            else if c_max == self.blue {
                hue = 60.0 * (((self.red - self.green) / delta) + 4.0);
            }
            else {
                return Err(format_err!("c_max matches neither R, G or B"))
            }
        }
        
        let lightness = (c_max + c_min) / 2.0;
        let mut saturation = 0.0;
        if delta != 0.0 {
            saturation = delta / (1.0 - ((2.0 * lightness) - 1.0).abs());
        }
        

        Ok(ColorHSLA {
            hue: hue,
            saturation: saturation,
            lightness: lightness,
            alpha: 1.0,
        })
    }

    fn min_3(f_1: f32, f_2: f32, f_3: f32) -> f32 {
        Self::min_2(Self::min_2(f_1, f_2), f_3)
    }

    fn min_2(f_1: f32, f_2: f32) -> f32 {
        if f_1 <= f_2 {
            return f_1
        }
        f_2
    }

    fn max_3(f_1: f32, f_2: f32, f_3: f32) -> f32 {
        Self::max_2(Self::max_2(f_1, f_2), f_3)
    }

    fn max_2(f_1: f32, f_2: f32) -> f32 {
        if f_1 >= f_2 {
            return f_1
        }
        f_2
    }

    fn decode_hex_color(chars: &mut Chars) -> Result<f32, Error> {
        let c_1 = chars.next().ok_or(format_err!("some err"))?;
        let c_2 = chars.next().ok_or(format_err!("some err"))?;
        let c_1 = Self::decode_char(c_1)?;
        let c_2 = Self::decode_char(c_2)?;
        let color = (c_1 as i32) * 16 + (c_2 as i32);
        let color = (color as f32) / 255.0;
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

pub struct ColorHSLA {
    pub hue: f32,
    pub saturation: f32,
    pub lightness: f32,
    pub alpha: f32,
}

impl ColorHSLA {
    pub fn to_rgb(&self) -> Result<ColorRGBA, Error> {
        Err(format_err!("not implemented"))
    }
}

#[cfg(test)]
mod tests {
    use super::ColorRGBA;

    #[test]
    fn parse_color_string() {
        let color_string = "#FF0077";
        let color = ColorRGBA::parse_string(&color_string).unwrap();
        assert_eq!(color.red, 1.0);
        assert_eq!(color.green, 0.0);
        assert_eq!(color.blue, 0.46666667);
        assert_eq!(color.alpha, 1.0);
    }

    #[test]
    fn rgba_to_hsla() {
        let rgba = ColorRGBA {
            red: 0.137254902,
            green: 0.784313725,
            blue: 0.784313725,
            alpha: 1.0,
        };
        let hsla = rgba.to_hsla().unwrap();
        assert_eq!(hsla.hue, 180.0);
        assert!(hsla.saturation > 0.69 && hsla.saturation < 0.71);
        assert!(hsla.lightness > 0.45 && hsla.lightness < 0.47);
        assert_eq!(hsla.alpha, 1.0);
    }
}