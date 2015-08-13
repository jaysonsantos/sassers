use sass::op::Op;

use std::fmt;
use std::borrow::Cow;

#[derive(Debug, Clone, PartialEq)]
pub enum ValuePart<'a> {
    Variable(Cow<'a, str>),
    String(Cow<'a, str>),
    Number(f32),
    Operator(Op),
}

impl<'a> fmt::Display for ValuePart<'a> {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match *self {
            ValuePart::Variable(ref name) => write!(f, "{}", name),
            ValuePart::String(ref str) => write!(f, "{}", str),
            ValuePart::Number(ref num) => write!(f, "{}", num),
            ValuePart::Operator(..) => unreachable!(),
        }

    }
}