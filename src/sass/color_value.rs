use error::{SassError, ErrorKind, Result};
use sass::op::Op;
use sass::number_value::NumberValue;

use std::borrow::Cow;
use std::fmt;
use std::cmp;

#[derive(Debug, Clone, PartialEq)]
pub struct ColorValue<'a> {
    pub red: i32,
    pub green: i32,
    pub blue: i32,
    pub original: Cow<'a, str>,
}

impl<'a, 'b> ColorValue<'a> {
    pub fn from_hex(hex: Cow<'a, str>) -> Result<ColorValue<'a>> {
        if hex.len() == 4 {
            Ok(ColorValue {
                red:   i32::from_str_radix(&hex[1..2], 16).unwrap() * 17,
                green: i32::from_str_radix(&hex[2..3], 16).unwrap() * 17,
                blue:  i32::from_str_radix(&hex[3..4], 16).unwrap() * 17,
                original: hex,
            })
        } else if hex.len() == 7 {
            Ok(ColorValue {
                red:   i32::from_str_radix(&hex[1..3], 16).unwrap(),
                green: i32::from_str_radix(&hex[3..5], 16).unwrap(),
                blue:  i32::from_str_radix(&hex[5..7], 16).unwrap(),
                original: hex,
            })
        } else {
            Err(SassError {
                kind: ErrorKind::InvalidColor,
                message: format!("Invalid hex color: {}", hex),
            })
        }
    }

    pub fn from_rgb(r: i32, g: i32, b: i32) -> ColorValue<'a> {
        ColorValue {
            red: r, green: g, blue: b, original: format!("#{:02x}{:02x}{:02x}", r, g, b).into(),
        }
    }

    pub fn into_owned(self) -> ColorValue<'b> {
        ColorValue {
            red: self.red, green: self.green, blue: self.blue,
            original: self.original.into_owned().into(),
        }
    }

    pub fn apply_math(self, op: Op, nv: NumberValue<'a>) -> Result<ColorValue<'a>> {
        Ok(ColorValue::from_rgb(
            cmp::min(try!(op.math(self.red as f32, nv.scalar)) as i32, 255),
            cmp::min(try!(op.math(self.green as f32, nv.scalar)) as i32, 255),
            cmp::min(try!(op.math(self.blue as f32, nv.scalar)) as i32, 255),
        ))
    }

    pub fn combine_colors(self, op: Op, c: ColorValue<'a>) -> Result<ColorValue<'a>> {
        Ok(ColorValue::from_rgb(
            try!(op.math(self.red as f32, c.red as f32)) as i32,
            try!(op.math(self.green as f32, c.green as f32)) as i32,
            try!(op.math(self.blue as f32, c.blue as f32)) as i32,
        ))
    }
}

impl<'a> fmt::Display for ColorValue<'a> {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        let candidate = format!("#{:02x}{:02x}{:02x}", self.red, self.green, self.blue);
        if candidate.len() < self.original.len() {
            write!(f, "{}", candidate)
        } else {
            write!(f, "{}", self.original)
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use sass::number_value::NumberValue;
    use sass::op::Op;
    use std::borrow::Cow::Borrowed;

    #[test]
    fn it_ignores_overflow_when_not_a_named_color() {
        let c = ColorValue::from_hex(Borrowed("#ff0000")).unwrap();
        let res = c.apply_math(Op::Plus, NumberValue::from_scalar(1.0)).unwrap();
        assert_eq!("#ff0101", format!("{}", res));
    }
}
