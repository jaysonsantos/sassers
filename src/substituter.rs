use error::{Result, SassError, ErrorKind};
use evaluator::Evaluator;
use event::Event;
use sass::rule::SassRule;
use sass::variable::SassVariable;
use sass::number_value::NumberValue;
use sass::value_part::ValuePart;
use sass::mixin::{SassMixin, SassMixinParameter, SassMixinArgument};

use std::collections::HashMap;
use std::borrow::Cow::{self, Borrowed, Owned};

pub struct Substituter<'vm, I> {
    tokenizer: I,
    variables: HashMap<String, ValuePart<'vm>>,
    mixins:    HashMap<String, SassMixin<'vm>>,
}

impl<'vm, I> Substituter<'vm, I> {
    pub fn new(tokenizer: I) -> Substituter<'vm, I> {
        Substituter {
            tokenizer: tokenizer,
            variables: HashMap::new(),
            mixins:    HashMap::new(),
        }
    }
}

impl<'a, I> Iterator for Substituter<'a, I>
    where I: Iterator<Item = Result<Event<'a>>>
{
    type Item = Result<Event<'a>>;

    fn next(&mut self) -> Option<Result<Event<'a>>> {
        match self.tokenizer.next() {
            Some(Ok(Event::Variable(SassVariable { name, value }))) => {
                let val = match owned_evaluated_value(value, &self.variables) {
                    Ok(v) => v,
                    Err(e) => return Some(Err(e)),
                };
                self.variables.insert((*name).to_string(), val);
                self.next()
            },
            Some(Ok(Event::Mixin(mixin))) => {
                self.mixins.insert((mixin.name).to_string(), mixin);
                self.next()
            },
            Some(Ok(Event::Rule(sass_rule))) => {
                let replaced = match replace_children_in_scope(
                    sass_rule.children, self.variables.clone(), self.mixins.clone()
                ) {
                    Ok(children) => children,
                    Err(e) => return Some(Err(e)),
                };

                Some(Ok(Event::Rule(SassRule {
                    children: replaced, ..sass_rule
                })))
            },
            Some(Ok(Event::MixinCall(..))) => unimplemented!(),
            other => other,
        }
    }
}

fn replace_children_in_scope<'b>(
    children: Vec<Event<'b>>,
    mut local_variables: HashMap<String, ValuePart<'b>>,
    local_mixins: HashMap<String, SassMixin<'b>>) -> Result<Vec<Event<'b>>> {

    let mut results = Vec::new();

    for c in children.into_iter() {
        match c {
            Event::Variable(SassVariable { name, value }) => {
                let val = try!(owned_evaluated_value(value, &local_variables));
                local_variables.insert((*name).to_string(), val);
            },
            Event::UnevaluatedProperty(name, value) => {
                let mut ev = Evaluator::new_from_string(&value);
                let ev_res = try!(ev.evaluate(&local_variables)).into_owned();

                results.push(Event::Property(
                    name,
                    ev_res,
                ));
            },
            Event::Rule(rule) => {
                let res = try!(replace_children_in_scope(
                    rule.children, local_variables.clone(), local_mixins.clone()
                ));
                results.push(Event::Rule(SassRule {
                    children: res, ..rule
                }));
            },
            Event::MixinCall(mixin_call) => {
                let mixin_name = mixin_call.name.into_owned();
                let mixin_definition = match local_mixins.get(&mixin_name) {
                    Some(mixin) => mixin,
                    None => return Err(SassError {
                        kind: ErrorKind::ExpectedMixin,
                        message: format!("Cannot find mixin named `{}`", mixin_name),
                    }),
                };

                let mixin_replacements = try!(collate_mixin_args(
                    &mixin_definition.parameters,
                    &mixin_call.arguments,
                ));

                let mut res = try!(replace_children_in_scope(
                    mixin_definition.children.clone(), mixin_replacements, local_mixins.clone()
                ));

                results.append(&mut res);
            },
            other => results.push(other),
        }
    }
    Ok(results)
}

fn collate_mixin_args<'a>(
    parameters: &Vec<SassMixinParameter<'a>>,
    arguments: &Vec<SassMixinArgument<'a>>) -> Result<HashMap<String, ValuePart<'a>>> {

    let mut named_arguments = HashMap::new();

    for a in arguments.iter() {
        match a.name {
            Some(ref name) => {
                named_arguments.insert(name.clone(), a.value.clone().into_owned());
            },
            None => {},
        }
    }

    let mut replacements = HashMap::new();

    for (i, p) in parameters.iter().enumerate() {
        let replacement_name = p.name.to_string();

        let replacement_value =
            try!(named_arguments.get(&p.name).and_then( |v| Some(v.to_string()) )
                .or(arguments.get(i).and_then( |a| Some(a.value.clone().into_owned()) ))
                .or(p.default.clone().and_then( |d| Some(d.into_owned()) ))
                .ok_or(SassError {
                    kind: ErrorKind::ExpectedMixinArgument,
                    message: format!("Cannot find argument for mixin parameter named `{}` in arguments `{:?}`", p.name, arguments),
                }));

        replacements.insert(replacement_name, ValuePart::String(Owned(replacement_value)));
    }

    Ok(replacements)
}

fn owned_evaluated_value<'a>(
    value: Cow<'a, str>,
    variables: &HashMap<String, ValuePart<'a>>) -> Result<ValuePart<'a>> {

    let value_part = match value {
        Cow::Borrowed(v) => {
            try!(Evaluator::new_from_string(&v).evaluate(variables))
        },
        Cow::Owned(v) => {
            try!(Evaluator::new_from_string(&v).evaluate(variables)).into_owned()
        },
    };
    Ok(match value_part {
        ValuePart::Number(nv) => ValuePart::Number(NumberValue { computed: true, ..nv }),
        other => other,
    })
}
