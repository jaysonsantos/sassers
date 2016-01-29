use sass::output_style::SassOutputStyle;
use sass::rule::SassRule;
use token::Lexeme;
use error::{Result};

use std::io::Write;

#[derive(Debug, Clone, PartialEq)]
pub enum Node {
    Rule(SassRule),
    Property(Lexeme, Lexeme),
}

impl Node {
    pub fn stream<W: Write>(&self, output: &mut W, style: SassOutputStyle) -> Result<()> {
        match *self {
            Node::Rule(ref sr) => try!(sr.stream(output, style)),
            Node::Property(ref name, ref value) => {
                let ref n = name.token;
                let ref v = value.token;
                // grumble mumble format strings you know they're a string literal
                let property = match style {
                    SassOutputStyle::Nested     => format!("  {}: {};", n, v),
                    SassOutputStyle::Compressed => format!("{}:{}", n, v),
                    SassOutputStyle::Expanded   => format!("  {}: {};", n, v),
                    SassOutputStyle::Compact    => format!("{}: {};", n, v),
                    SassOutputStyle::Debug      => format!("{:?}\n", self),
                    _ => unreachable!(),
                };
                try!(write!(output, "{}", property));
            },
        }
        Ok(())
    }
}
