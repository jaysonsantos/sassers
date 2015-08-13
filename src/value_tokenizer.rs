use sass::value_part::ValuePart;

use std::borrow::Cow::Borrowed;

#[derive(Debug)]
pub struct ValueTokenizer<'a> {
    value_str: &'a str,
    bytes: &'a [u8],
    offset: usize,
}

impl<'a> ValueTokenizer<'a> {
    pub fn new(value_str: &'a str) -> ValueTokenizer<'a> {
        ValueTokenizer {
            value_str: &value_str,
            bytes: &value_str.as_bytes(),
            offset: 0,
        }
    }

    pub fn parse(&mut self) -> Option<ValuePart<'a>> {
        self.skip_leading_whitespace();

        let start = self.offset;
        let mut i = self.offset;
        let limit = self.value_str.len();

        if is_operator(self.bytes[start]) {
            self.offset = start + 1;
            Some(ValuePart::Operator(self.value_str[start..start + 1].parse().unwrap()))
        } else if is_number(self.bytes[start]) {
            i += self.scan_while(&self.value_str[i..limit], is_number);
            self.offset = i;
            Some(ValuePart::Number(self.value_str[start..i].parse().unwrap()))
        } else if self.bytes[start] == b'$' {
            i += self.scan_while(&self.value_str[i..limit], isnt_space);
            self.offset = i;
            Some(ValuePart::Variable(Borrowed(&self.value_str[start..i])))
        } else {
            i += self.scan_while(&self.value_str[i..limit], isnt_space);
            self.offset = i;
            Some(ValuePart::String(Borrowed(&self.value_str[start..i])))
        }
    }

    fn scan_while<F>(&mut self, data: &str, f: F) -> usize
            where F: Fn(u8) -> bool {
        match data.as_bytes().iter().position(|&c| !f(c)) {
            Some(i) => i,
            None => data.len()
        }
    }

    fn skip_leading_whitespace(&mut self) {
       let mut i = self.offset;
       let limit = self.value_str.len();

       while i < limit {
           let c = self.bytes[i];
           if is_space(c) {
               i += self.scan_while(&self.value_str[i..limit], is_space);
           } else {
               self.offset = i;
               return
           }
       }
       self.offset = limit;
   }
}

impl<'a> Iterator for ValueTokenizer<'a> {
    type Item = ValuePart<'a>;

    fn next(&mut self) -> Option<ValuePart<'a>> {
        if self.offset < self.value_str.len() {
            return self.parse()
        }
        None
    }
}

fn is_space(c: u8) -> bool {
    c == b' '
}

fn isnt_space(c: u8) -> bool {
    !is_space(c)
}

fn is_number(c: u8) -> bool {
    let result = match c {
        b'0' ... b'9' | b'.' => true,
        _ => false,
    };
    result
}

fn is_operator(c: u8) -> bool {
    match c {
        b'+' | b'-' | b'*' | b'/' | b'%' | b'(' | b')' | b',' => true,
        _ => false,
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use sass::value_part::ValuePart;
    use sass::op::Op;
    use std::borrow::Cow::Borrowed;

    #[test]
    fn it_returns_string_part() {
        let mut vt = ValueTokenizer::new("foo");
        assert_eq!(Some(ValuePart::String(Borrowed(&"foo"))), vt.next());
        assert_eq!(None, vt.next());
    }

    #[test]
    fn it_returns_space_separated_string_parts() {
        let mut vt = ValueTokenizer::new("foo bar");
        assert_eq!(Some(ValuePart::String(Borrowed(&"foo"))), vt.next());
        assert_eq!(Some(ValuePart::String(Borrowed(&"bar"))), vt.next());
        assert_eq!(None, vt.next());
    }

    #[test]
    fn it_returns_variable() {
        let mut vt = ValueTokenizer::new("$foo");
        assert_eq!(Some(ValuePart::Variable(Borrowed(&"$foo"))), vt.next());
        assert_eq!(None, vt.next());
    }

    #[test]
    fn it_returns_variables_and_string_parts() {
        let mut vt = ValueTokenizer::new("foo $bar baz $quux");
        assert_eq!(Some(ValuePart::String(Borrowed(&"foo"))), vt.next());
        assert_eq!(Some(ValuePart::Variable(Borrowed(&"$bar"))), vt.next());
        assert_eq!(Some(ValuePart::String(Borrowed(&"baz"))), vt.next());
        assert_eq!(Some(ValuePart::Variable(Borrowed(&"$quux"))), vt.next());
        assert_eq!(None, vt.next());
    }

    #[test]
    fn it_returns_number() {
        let mut vt = ValueTokenizer::new("3");
        assert_eq!(Some(ValuePart::Number(3.0)), vt.next());
        assert_eq!(None, vt.next());
    }

    #[test]
    fn it_returns_two_numbers() {
        let mut vt = ValueTokenizer::new("3 8.9");
        assert_eq!(Some(ValuePart::Number(3.0)), vt.next());
        assert_eq!(Some(ValuePart::Number(8.9)), vt.next());
        assert_eq!(None, vt.next());
    }

    #[test]
    fn it_returns_operator() {
        let mut vt = ValueTokenizer::new("+");
        assert_eq!(Some(ValuePart::Operator(Op::Plus)), vt.next());
        assert_eq!(None, vt.next());
    }

    #[test]
    fn it_returns_numbers_and_operators() {
        let mut vt = ValueTokenizer::new("6 + 75.2");
        assert_eq!(Some(ValuePart::Number(6.0)), vt.next());
        assert_eq!(Some(ValuePart::Operator(Op::Plus)), vt.next());
        assert_eq!(Some(ValuePart::Number(75.2)), vt.next());
        assert_eq!(None, vt.next());
    }

    #[test]
    fn it_does_stuff_with_parens() {
        let mut vt = ValueTokenizer::new("2+(3 4)");
        assert_eq!(Some(ValuePart::Number(2.0)), vt.next());
        assert_eq!(Some(ValuePart::Operator(Op::Plus)), vt.next());
        assert_eq!(Some(ValuePart::Operator(Op::LeftParen)), vt.next());
        assert_eq!(Some(ValuePart::Number(3.0)), vt.next());
        assert_eq!(Some(ValuePart::Number(4.0)), vt.next());
        assert_eq!(Some(ValuePart::Operator(Op::RightParen)), vt.next());
        assert_eq!(None, vt.next());
    }
}
