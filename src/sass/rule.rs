use event::Event;
use sass::selector::SassSelector;
use sass::output_style::SassOutputStyle;
use error::Result;

use std::fmt;
use std::io::Write;

#[derive(Clone)]
pub struct SassRule {
    pub selectors: Vec<SassSelector>,
    pub children: Vec<Event>,
}

impl SassRule {
    pub fn new() -> SassRule {
        SassRule {
            selectors: Vec::new(),
            children: Vec::new(),
        }
    }

    fn selector_distribution(&self, parents: &str, separator: &str) -> String {
        match parents.len() {
            0 => self.selectors.iter().map(|s| s.name.value.clone()).collect::<Vec<_>>().join(separator),
            _ => parents.split(",").map(|p| {
                self.selectors.iter().map(|s| {
                    if s.name.value.contains("&") {
                        s.name.value.replace("&", p.trim())
                    } else {
                        format!("{} {}", p.trim(), s.name.value)
                    }
                }).collect::<Vec<_>>().join(separator)
            }).collect::<Vec<_>>().join(separator),
        }
    }

    pub fn stream<W: Write>(&self, output: &mut W, style: SassOutputStyle) -> Result<()> {
        try!(self.recursive_stream(output, style, "", ""));
        Ok(try!(write!(output, "{}", style.rule_separator())))
    }

    pub fn recursive_stream<W: Write>(&self, output: &mut W, style: SassOutputStyle, parents: &str, nesting: &str) -> Result<()> {
        let mut selector_string = self.selector_distribution(parents, &style.selector_separator());
        if style == SassOutputStyle::Compressed {
            selector_string = compress_selectors(selector_string);
        }

        let mut properties = self.children.iter().filter(|c| !c.is_child_rule() ).filter_map(|c| {
            let s = c.to_string(style);
            if s.len() > 0 { Some(s) } else { None }
        });
        let mut has_properties = false;

        // TODO: peek?
        if let Some(prop) = properties.next() {
            has_properties = true;
            try!(write!(output, "{}{}{{{}{}{}",
              selector_string,
              style.selector_brace_separator(),
              style.brace_property_separator(),
              style.before_property(nesting),
              prop,
            ));
            for prop in properties {
                try!(write!(output, "{}{}{}",
                    style.after_property(),
                    style.before_property(nesting),
                    prop
                ));
            }
            try!(write!(output, "{}}}", style.property_brace_separator()));
        }

        let mut child_rules = self.children.iter().filter(|c| c.is_child_rule() ).map(|c| {
            match c {
                &Event::Rule(ref rule) => rule,
                _ => unreachable!(),
            }
        });

        if let Some(child_rule) = child_rules.next() {
            let mut recursive_nesting = String::from(nesting);
            if has_properties {
                recursive_nesting.push_str("  ");
                try!(write!(output, "{}", style.rule_and_child_rules_separator(&recursive_nesting)));
            }
            try!(child_rule.recursive_stream(output, style, &selector_string, &recursive_nesting));
            for child_rule in child_rules {
                try!(write!(output, "{}", style.child_rule_separator(has_properties)));
                try!(child_rule.recursive_stream(output, style, &selector_string, &recursive_nesting));
            }
        }
        Ok(())
    }
}

fn compress_selectors(selector_string: String) -> String {
    selector_string.replace(" > ", ">").replace(" + ", "+")
}

impl fmt::Debug for SassRule {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        let children = self.children.iter().map(|c| format!("{:?}", c)).collect::<Vec<_>>().join("\n");
        let indented_children = children.split("\n").collect::<Vec<_>>().join("\n  ");
        write!(f, "{:?} {{\n  {}\n}}", self.selectors, indented_children)
    }
}
