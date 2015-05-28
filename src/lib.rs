mod token;

pub fn compile(sass: String, style: &str) -> String {
    let mut sr = SassTokenizer::new(sass);
    println!("{:?}", sr);
    let parsed = sr.parse();
    match style {
        "nested"     => nested(parsed),
        "compressed" => compressed(parsed),
        "expanded"   => expanded(parsed),
        "compact"    => compact(parsed),
        _            => panic!("Unknown style: {}. Please specify one of nested, compressed, expanded, or compact.", style),
    }
}

fn nested(sass: String) -> String {
    sass
}

fn compressed(sass: String) -> String {
    sass.replace(" ", "").replace("\n", "")
}

fn expanded(sass: String) -> String {
    sass
}

fn compact(sass: String) -> String {
    sass
}

struct SassRuleSet {
    pub rules: Vec<SassRule>,
}

struct SassRule {
    pub selectors: Vec<token::Range>,
    pub propsNVals: Vec<PropertyValueSet>,
}

struct PropertyValueSet {
    pub property: token::Range,
    pub value: token::Range,
}

#[derive(Debug)]
struct SassTokenizer {
    pub pos: u32,
    pub last_pos: u32,
    pub curr: Option<char>,
    pub peek_tok: token::Range,
    sass: String,
}

impl SassTokenizer {
    pub fn new(str: String) -> SassTokenizer {
        let mut sr = SassTokenizer {
            pos: 0,
            last_pos: 0,
            curr: Some('\n'),
            peek_tok: token::Range { start_pos: 0, end_pos: 0, token: token::Eof },
            sass: str,
        };
        sr.bump();
        sr.advance_token();
        sr
    }

    pub fn bump(&mut self) {
        self.last_pos = self.pos;
        let current_pos = self.pos as usize;
        if current_pos < self.sass.len() {
            let ch = char_at(&self.sass, current_pos);

            self.pos = self.pos + (ch.len_utf8() as u32);
            self.curr = Some(ch);
        } else {
            self.curr = None;
        }
    }

    pub fn advance_token(&mut self) {
        match self.scan_whitespace() {
            Some(whitespace) => {
                self.peek_tok = whitespace;
            },
            None => {
                if self.is_eof() {
                    self.peek_tok = token::Range { start_pos: self.pos + 1, end_pos: self.pos + 1, token: token::Eof };
                } else {
                    self.peek_tok = self.next_token_inner();
                }
            },
        }
    }

    pub fn next_token(&mut self) -> token::Range {
        let ret_val = self.peek_tok.clone();
        self.advance_token();
        ret_val
    }

    pub fn parse(&mut self) -> String {
        println!("{:?}", self.next_token());
        println!("{:?}", self.next_token());
        println!("{:?}", self.next_token());
        println!("{:?}", self.next_token());
        println!("{:?}", self.next_token());
        println!("{:?}", self.next_token());
        println!("{:?}", self.next_token());
        println!("{:?}", self.next_token());
        println!("{:?}", self.next_token());
        println!("{:?}", self.next_token());
        println!("{:?}", self.next_token());
        println!("{:?}", self.next_token());
        self.sass.clone()
    }

    fn parse_rules(&mut self) -> Result<SassRuleSet, &'static str> {
        let mut rules = vec![];
        while let Some(rule) = try!(self.parse_rule()) {
            rules.push(rule);
        }
        Ok(SassRuleSet { rules: rules })
    }

    fn parse_rule(&mut self) -> Result<Option<SassRule>, &'static str> {
        let mut selectors = vec![];
        while let Some(selector) = try!(self.parse_selector()) {
            selectors.push(selector);
        }
        let mut propsNVals = vec![];
        while let Some(property_value_set) = try!(self.parse_property_value_set()) {
            propsNVals.push(property_value_set);
        }
        Ok(Some(SassRule { selectors: selectors, propsNVals: propsNVals }))
    }

    fn parse_selector(&mut self) -> Result<Option<token::Range>, &'static str> {
        Err("not implemented")
    }

    fn parse_property_value_set(&mut self) -> Result<Option<PropertyValueSet>, &'static str> {
        Err("not implemented")
    }

    fn next_token_inner(&mut self) -> token::Range {
        let c = match self.curr {
            Some(c) => c,
            None => return token::Range { start_pos: self.pos + 1, end_pos: self.pos + 1, token: token::Eof },
        };

        if c >= 'a' && c <= 'z' {
            let start = self.last_pos;
            while !self.curr.is_none() && self.curr.unwrap() >= 'a' && self.curr.unwrap() <= 'z' {
                self.bump();
            }
            return token::Range { start_pos: start, end_pos: self.last_pos, token: token::Text }
        }

        match c {
            ';' => { self.bump(); return token::Range { start_pos: self.last_pos, end_pos: self.last_pos, token: token::Semi }; },
            ':' => { self.bump(); return token::Range { start_pos: self.last_pos, end_pos: self.last_pos, token: token::Colon }; },
            ',' => { self.bump(); return token::Range { start_pos: self.last_pos, end_pos: self.last_pos, token: token::Comma }; },
            '{' => { self.bump(); return token::Range { start_pos: self.last_pos, end_pos: self.last_pos, token: token::OpenDelim(token::Brace) }; },
            '}' => { self.bump(); return token::Range { start_pos: self.last_pos, end_pos: self.last_pos, token: token::CloseDelim(token::Brace) }; },
            _   => { return token::Range { start_pos: self.last_pos, end_pos: self.last_pos, token: token::Unknown} },
        }
    }

    fn scan_whitespace(&mut self) -> Option<token::Range> {
        match self.curr.unwrap_or('\0') {
            c if is_whitespace(Some(c)) => {
                let start = self.last_pos;
                while is_whitespace(self.curr) { self.bump(); }
                Some(token::Range { start_pos: start, end_pos: self.last_pos, token: token::Whitespace })
            },
            _ => None
        }
    }

    fn is_eof(&self) -> bool {
        self.curr.is_none()
    }
}

pub fn char_at(s: &str, byte: usize) -> char {
    s[byte..].chars().next().unwrap()
}

pub fn is_whitespace(c: Option<char>) -> bool {
    match c.unwrap_or('\x00') { // None can be null for now... it's not whitespace
        ' ' | '\n' | '\t' | '\r' => true,
        _ => false
    }
}
