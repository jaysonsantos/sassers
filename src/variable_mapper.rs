use evaluator::Evaluator;
use event::Event;
use top_level_event::TopLevelEvent;
use sass::rule::SassRule;
use sass::variable::SassVariable;
use std::collections::HashMap;

pub struct VariableMapper<I> {
    tokenizer: I,
    variables: HashMap<String, String>,
}

impl<I> VariableMapper<I> {
    pub fn new(tokenizer: I) -> VariableMapper<I> {
        VariableMapper {
            tokenizer: tokenizer,
            variables: HashMap::new(),
        }
    }

    fn replace_children_in_scope<'b>(&self, children: Vec<Event<'b>>, mut local_variables: HashMap<String, String>) -> Vec<Event<'b>> {
        children.into_iter().filter_map(|c|
            match c {
                Event::Variable(SassVariable { name, value }) => {
                    let val = Evaluator::new_from_string(
                        &value, &local_variables
                    ).evaluate().to_string();
                    local_variables.insert((*name).to_string(), val);
                    None
                },
                Event::Property(name, value) => {
                    Some(Event::Property(
                        name,
                        Evaluator::new_from_string(
                            &value, &local_variables
                        ).evaluate().to_string().into()
                    ))
                },
                Event::ChildRule(rule) => {
                    Some(Event::ChildRule(SassRule {
                        children: self.replace_children_in_scope(
                            rule.children, local_variables.clone()
                        ), ..rule
                    }))
                },
                other => Some(other)
            }
        ).collect::<Vec<_>>()
    }
}

impl<'a, I> Iterator for VariableMapper<I>
    where I: Iterator<Item = TopLevelEvent<'a>>
{
    type Item = TopLevelEvent<'a>;

    fn next(&mut self) -> Option<TopLevelEvent<'a>> {
        match self.tokenizer.next() {
            Some(TopLevelEvent::Variable(SassVariable { name, value })) => {
                let val = Evaluator::new_from_string(
                    &value, &self.variables
                ).evaluate().to_string();
                self.variables.insert((*name).to_string(), val);
                self.next()
            },
            Some(TopLevelEvent::Rule(sass_rule)) => {
                Some(TopLevelEvent::Rule(SassRule {
                    children: self.replace_children_in_scope(
                        sass_rule.children, self.variables.clone()
                    ), ..sass_rule
                }))
            },
            other => other,
        }
    }
}
