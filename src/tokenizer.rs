use error::{SassError, ErrorKind, Result};
use event::Event;
use sass::comment::SassComment;
use sass::rule::SassRule;
use sass::selector::SassSelector;
use sass::variable::SassVariable;
use sass::mixin::{SassMixin, SassMixinCall, SassMixinArgument};
use top_level_event::TopLevelEvent;
use tokenizer_utils::*;

use std::borrow::Cow::{self, Borrowed};

#[derive(Debug)]
pub struct Tokenizer<'a> {
    toker: Toker<'a>,
}

#[derive(PartialEq, Debug, Copy, Clone)]
enum State {
    InComment,
    InSelectors,
    InProperties,
}

#[derive(Debug)]
pub struct InnerTokenizer<'a> {
    toker: Toker<'a>,
    state: State,
}

impl<'a> InnerTokenizer<'a> {

    fn limit(&self) -> usize {
        self.toker.limit()
    }

    fn start_something(&mut self) -> Result<Option<Event<'a>>> {
        self.toker.skip_leading_whitespace();

        if self.toker.at_eof() {
            return Ok(None)
        }

        let c = self.toker.bytes[self.toker.offset];

        if c == b'}' {
            return Ok(None)
        }
        if c == b'/' && (self.toker.offset + 1) < self.limit() {
            let d = self.toker.bytes[self.toker.offset + 1];
            if d == b'*' {
                return self.next_comment()
            }
        }

        match self.state {
            State::InProperties => self.next_property(),
            State::InSelectors => self.next_rule(),
            other => unreachable!("got {:?}", other),
        }
    }

    fn next_rule(&mut self) -> Result<Option<Event<'a>>> {
        debug!("in next rule, offset {:?}", self.toker.offset);
        let mut current_sass_rule = SassRule::new();
        current_sass_rule.selectors = try!(self.selector_list());
        let mut inner = InnerTokenizer {
            toker: Toker {
                inner_str: &self.toker.inner_str,
                bytes: &self.toker.bytes,
                offset: self.toker.offset,
            },
            state: State::InProperties,
        };
        while let Some(Ok(e)) = inner.next() {
            current_sass_rule.children.push(e);
        }
        self.toker.offset = inner.toker.offset;
        debug!("returned from inner rule, offset {:?}", self.toker.offset);
        self.state = State::InProperties;

        while let Some(Ok(e)) = self.next() {
            current_sass_rule.children.push(e);
        }
        try!(self.toker.eat("}"));

        Ok(Some(Event::ChildRule(current_sass_rule)))
    }

    fn next_comment(&mut self) -> Result<Option<Event<'a>>> {
        let comment_body_beginning = self.toker.offset;
        let mut i = comment_body_beginning + 2;

        while i < self.limit() {
            i += self.toker.scan_while_or_end(i, isnt_asterisk);
            self.toker.offset = i;

            if self.toker.eat("*/").is_ok() {
                return Ok(Some(
                    Event::Comment(Borrowed(
                        &self.toker.inner_str[comment_body_beginning..self.toker.offset]
                    ))
                ))
            } else {
                i += 1;
            }
        }
        self.toker.offset = self.limit();
        Err(SassError {
            kind: ErrorKind::UnexpectedEof,
            message: String::from(
                "Expected comment; reached EOF instead."
            ),
        })
    }

    fn next_property(&mut self) -> Result<Option<Event<'a>>> {
        self.toker.skip_leading_whitespace();

        if self.toker.at_eof() {
            return Ok(None)
        }

        let c = self.toker.curr_byte();
        if c == b'}' {
            return Ok(None)
        }

        let d = self.toker.next_byte();
        if c == b'/' && d == b'*' {
            self.state = State::InComment;
            return Ok(None)
        }

        let saved_offset = self.toker.offset;

        if self.toker.eat("@include ").is_ok() {
            return self.next_mixin_call()
        }

        if self.toker.eat("@extend ").is_ok() {
            return self.next_mixin_call()
        }

        let prop_name = try!(self.toker.next_name());

        let c = self.toker.curr_byte();
        if c == b'{' {
            self.state = State::InSelectors;
            self.toker.offset = saved_offset;
            return match self.next() {
                Some(Ok(e))  => Ok(Some(e)),
                Some(Err(e)) => Err(e),
                None         => Ok(None),
            }
        }

        try!(self.toker.eat(":"));
        self.toker.skip_leading_whitespace();

        let prop_value = try!(self.toker.next_value());

        try!(self.toker.eat(";"));
        self.toker.skip_leading_whitespace();

        if prop_name.as_bytes()[0] == b'$' {
            Ok(Some(Event::Variable(SassVariable {
                name:  prop_name,
                value: prop_value,
            })))
        } else {
            Ok(Some(Event::UnevaluatedProperty(
                prop_name,
                prop_value,
            )))
        }
    }

    fn next_mixin_call(&mut self) -> Result<Option<Event<'a>>> {
        self.toker.skip_leading_whitespace();
        let name_beginning = self.toker.offset;
        let mut i = name_beginning;

        i += self.toker.scan_while_or_end(i, valid_name_char);
        let name_end = i;
        let name = Borrowed(&self.toker.inner_str[name_beginning..name_end]);

        self.toker.offset = i;

        let arguments = if self.toker.eat("(").is_ok() {
            try!(self.toker.tokenize_list(",", ")", &valid_mixin_arg_char))
        } else {
            Vec::new()
        };

        try!(self.toker.eat(";"));

        let mixin_call = Event::MixinCall(SassMixinCall {
            name: name,
            arguments: arguments,
        });

        return Ok(Some(mixin_call))
    }

    fn selector_list(&mut self) -> Result<Vec<SassSelector<'a>>> {
        let selectors = try!(self.toker.tokenize_list(",", "{", &valid_selector_char));
        self.state = State::InProperties;

        Ok(selectors.into_iter().map(|s| SassSelector::new(s)).collect())
    }
}

impl<'a> Iterator for InnerTokenizer<'a> {
    type Item = Result<Event<'a>>;

    fn next(&mut self) -> Option<Result<Event<'a>>> {
        if !self.toker.at_eof() {
            return match self.start_something() {
                Ok(Some(t)) => Some(Ok(t)),
                Ok(None) => None,
                Err(e) => Some(Err(e)),
            }
        }
        None
    }
}


impl<'a> Tokenizer<'a> {
    pub fn new(inner_str: &'a str) -> Tokenizer<'a> {
        Tokenizer {
            toker: Toker {
                inner_str: &inner_str,
                bytes: &inner_str.as_bytes(),
                offset: 0,
            },
        }
    }

    fn limit(&self) -> usize {
        self.toker.limit()
    }

    fn start_something(&mut self) -> Result<Option<TopLevelEvent<'a>>> {
        self.toker.skip_leading_whitespace();

        if self.toker.at_eof() {
            return Ok(None)
        }

        let c = self.toker.bytes[self.toker.offset];

        if c == b'$' {
            return self.next_variable()
        }

        if self.toker.eat("@mixin ").is_ok() {
            return self.next_mixin()
        }

        if self.toker.eat("@include ").is_ok() {
            return self.next_mixin_call()
        }

        if c == b'/' && (self.toker.offset + 1) < self.limit() {
            let d = self.toker.bytes[self.toker.offset + 1];
            if d == b'*' {
                return self.next_comment()
            }
        }

        let mut inner = InnerTokenizer {
            toker: Toker {
                inner_str: &self.toker.inner_str,
                bytes: &self.toker.bytes,
                offset: self.toker.offset,
            },
            state: State::InSelectors,
        };
        let ret = match inner.next_rule() {
            Ok(Some(Event::ChildRule(rule))) => Ok(Some(TopLevelEvent::Rule(rule))),
            other => return Err(SassError {
                kind: ErrorKind::TokenizerError,
                message: format!(
                    "Expected sass rule from inner tokenizer, got: {:?}.",
                    other
                ),
            }),
        };
        self.toker.offset = inner.toker.offset;
        return ret
    }

    fn next_comment(&mut self) -> Result<Option<TopLevelEvent<'a>>> {
        let comment_body_beginning = self.toker.offset;
        let mut i = comment_body_beginning + 2;

        while i < self.limit() {
            i += self.toker.scan_while_or_end(i, isnt_asterisk);
            self.toker.offset = i;

            if self.toker.eat("*/").is_ok() {
                return Ok(Some(
                    TopLevelEvent::Comment(SassComment { comment: Event::Comment(Borrowed(
                        &self.toker.inner_str[comment_body_beginning..self.toker.offset]
                    ))})
                ))
            } else {
                i += 1;
            }
        }
        self.toker.offset = self.limit();
        Err(SassError {
            kind: ErrorKind::UnexpectedEof,
            message: String::from(
                "Expected comment; reached EOF instead."
            ),
        })
    }

    fn next_variable(&mut self) -> Result<Option<TopLevelEvent<'a>>> {
        let var_name = try!(self.toker.next_name());

        try!(self.toker.eat(":"));
        self.toker.skip_leading_whitespace();

        let var_value = try!(self.toker.next_value());

        try!(self.toker.eat(";"));
        self.toker.skip_leading_whitespace();

        Ok(Some(TopLevelEvent::Variable(SassVariable {
            name:  var_name,
            value: var_value,
        })))
    }

    fn next_mixin(&mut self) -> Result<Option<TopLevelEvent<'a>>> {
        let name_beginning = self.toker.offset;
        let mut i = name_beginning;

        while i < self.limit() {
            i += self.toker.scan_while_or_end(i, valid_name_char);
            let name_end = i;

            self.toker.offset = i;
            try!(self.toker.eat("("));

            let arguments = try!(self.toker.tokenize_list(",", ")", &valid_mixin_arg_char));

            self.toker.skip_leading_whitespace();
            try!(self.toker.eat("{"));
            self.toker.skip_leading_whitespace();
            i = self.toker.offset;
            let body_beginning = i;
            i += self.toker.scan_while_or_end(i, isnt_end_curly_brace);
            let body_end = i;
            self.toker.offset = i;
            try!(self.toker.eat("}"));

            let mixin = TopLevelEvent::Mixin(SassMixin {
                name: Borrowed(&self.toker.inner_str[name_beginning..name_end]),
                arguments: arguments.into_iter().map(|a|
                    SassMixinArgument::new(a)
                ).collect(),
                body: Borrowed(&self.toker.inner_str[body_beginning..body_end]),
            });

            return Ok(Some(mixin))
        }
        self.toker.offset = self.limit();
        Err(SassError {
            kind: ErrorKind::UnexpectedEof,
            message: String::from(
                "Expected mixin declaration; reached EOF instead."
            ),
        })
    }

    fn next_mixin_call(&mut self) -> Result<Option<TopLevelEvent<'a>>> {
        self.toker.skip_leading_whitespace();
        let name_beginning = self.toker.offset;
        let mut i = name_beginning;

        while i < self.limit() {
            i += self.toker.scan_while_or_end(i, valid_name_char);
            let name_end = i;
            let name = Borrowed(&self.toker.inner_str[name_beginning..name_end]);

            self.toker.offset = i;

            let arguments = if self.toker.eat("(").is_ok() {
                try!(self.toker.tokenize_list(",", ")", &valid_mixin_arg_char))
            } else {
                Vec::new()
            };

            try!(self.toker.eat(";"));

            let mixin_call = TopLevelEvent::MixinCall(SassMixinCall {
                name: name,
                arguments: arguments,
            });

            return Ok(Some(mixin_call))

        }
        self.toker.offset = self.limit();
        Err(SassError {
            kind: ErrorKind::UnexpectedEof,
            message: String::from(
                "Expected mixin call; reached EOF instead."
            ),
        })
    }
}

impl<'a> Iterator for Tokenizer<'a> {
    type Item = Result<TopLevelEvent<'a>>;

    fn next(&mut self) -> Option<Result<TopLevelEvent<'a>>> {
        if !self.toker.at_eof() {
            return match self.start_something() {
                Ok(Some(t)) => Some(Ok(t)),
                Ok(None) => None,
                Err(e) => Some(Err(e)),
            }
        }
        None
    }
}
