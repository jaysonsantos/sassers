use std::borrow::Cow::*;
use std::borrow::Cow;
use std::fmt;

#[derive(Debug, Clone, PartialEq)]
pub struct NumberValue<'a> {
    pub scalar:   f32,
    pub unit:     Option<Cow<'a, str>>,
    pub computed: bool,
}

impl<'a, 'b> NumberValue<'a> {
    pub fn from_scalar(num: f32) -> NumberValue<'a> {
        NumberValue {
            scalar:   num,
            unit:     None,
            computed: false,
        }
    }

    #[cfg(test)]
    pub fn computed(num: f32) -> NumberValue<'a> {
        NumberValue {
            scalar:   num,
            unit:     None,
            computed: true,
        }
    }

    pub fn with_units(num: f32, unit: Cow<'a, str>) -> NumberValue<'a> {
        NumberValue {
            scalar:   num,
            unit:     Some(unit),
            computed: false,
        }
    }

    pub fn into_owned(self) -> NumberValue<'b> {
        NumberValue {
            scalar: self.scalar,
            unit: match self.unit {
                Some(u) => Some(u.into_owned().into()),
                None => None,
            },
            computed: self.computed,
        }
    }

    pub fn unit_string(&self) -> &str {
        match self.unit {
            Some(Borrowed(ref b)) => b,
            Some(Owned(ref o))    => o,
            None                  => "",
        }
    }
}

impl<'a> fmt::Display for NumberValue<'a> {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "{}{}", self.scalar, self.unit_string())
    }
}
