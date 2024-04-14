use biome_analyze::{context::RuleContext, declare_rule, Ast, Rule, RuleDiagnostic};
use biome_js_syntax::{jsx_ext::AnyJsxElement, AnyJsxAttribute};
use biome_rowan::TextRange;

declare_rule! {
    /// TODO: Succinct description of the rule.
    ///
    /// Require CSS properties in the style prop to be valid and kebab-cased (ex. 'font-size'),
    /// not camel-cased (ex. 'fontSize') like in React,
    /// and that property values with dimensions are strings,
    /// not numbers with implicit 'px' units.
    ///
    /// https://github.com/solidjs-community/eslint-plugin-solid/blob/main/docs/style-prop.md
    ///
    /// ## Examples
    ///
    /// ### Invalid
    ///
    /// ```js,expect_diagnostic
    /// <div style={{ fontSize: "10px" }}>Hello, world!</div>
    /// ```
    ///
    /// ### Valid
    ///
    /// ```js
    /// <div style={{ "font-size": "10px" }}>Hello, world!</div>
    /// ```
    ///
    pub StyleProp {
        version: "next",
        name: "styleProp",
        // TODO: eslint source solid
        recommended: false,
    }
}

fn is_kebab_case(input: &str) -> bool {
    if input.is_empty() {
        return false;
    }

    for c in input.chars() {
        if !(c.is_ascii_lowercase() || c.is_ascii_digit() || c == '-') {
            return false;
        }
    }

    true
}

// TODO: check lengthPercentageRegex = /\b(?:width|height|margin|padding|border-width|font-size)\b/i;

const PROPS_THAT_NEEDS_UNIT: &[&str] = &[
    "width",
    "height",
    "margin",
    "padding",
    "border-width",
    "font-size",
];

fn prop_needs_unit(name: &str) -> bool {
    PROPS_THAT_NEEDS_UNIT.contains(&name)
}

pub enum StylePropError {
    Kebab(String),
    NumberLiteral,
}

impl std::fmt::Display for StylePropError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::Kebab(name) => write!(f, "{name} is not kebab-case."),
            Self::NumberLiteral => {
                write!(f, "This CSS property value should be a string with a unit; Solid does not automatically append a \"px\" unit.")
            }
        }
    }
}

impl Rule for StyleProp {
    type Query = Ast<AnyJsxElement>;
    type State = Vec<(TextRange, StylePropError)>;
    type Signals = Option<Self::State>;
    type Options = ();

    fn run(ctx: &RuleContext<Self>) -> Self::Signals {
        use biome_js_syntax::AnyJsxAttributeValue::JsxExpressionAttributeValue as AttributeValue;

        let node = ctx.query();

        let mut invalid_style_attribute = vec![];

        for attribute in node.attributes() {
            if let AnyJsxAttribute::JsxAttribute(attr) = attribute {
                if let Ok(name) = attr.name() {
                    if name.to_string() == "style" {
                        if let AttributeValue(maybe_obj) =
                            attr.initializer().unwrap().value().unwrap()
                        {
                            if let Some(obj) =
                                maybe_obj.expression().unwrap().as_js_object_expression()
                            {
                                for member in obj.members().into_iter().flatten() {
                                    if let Some(member) = member.as_js_property_object_member() {
                                        if let Ok(name_token) = member.name() {
                                            let name = name_token.name().unwrap().to_string();

                                            if !is_kebab_case(&name) {
                                                let name_span = name_token
                                                    .as_js_literal_member_name()
                                                    .unwrap()
                                                    .value()
                                                    .unwrap()
                                                    .text_range();

                                                invalid_style_attribute.push((
                                                    name_span,
                                                    StylePropError::Kebab(name.clone()),
                                                ));
                                            }

                                            if prop_needs_unit(&name) {
                                                if let Ok(value) = member.value() {
                                                    // TODO: need to check negative numbers...

                                                    if let Some(literal) =
                                                        value.as_any_js_literal_expression()
                                                    {
                                                        if let Some(num) = literal
                                                            .as_js_number_literal_expression()
                                                            .and_then(|x| x.as_number())
                                                        {
                                                            if num != 0.0 {
                                                                invalid_style_attribute.push((
                                                                    literal
                                                                        .value_token()
                                                                        .unwrap()
                                                                        .text_range(),
                                                                    StylePropError::NumberLiteral,
                                                                ));
                                                            }
                                                        }
                                                    }
                                                }
                                            }
                                        };
                                    }
                                }
                            }
                        };
                    }
                }
            }
        }

        Some(invalid_style_attribute)
    }

    fn diagnostic(_: &RuleContext<Self>, issues: &Self::State) -> Option<RuleDiagnostic> {
        let mut iter = issues.iter();

        if let Some((range, err)) = iter.next() {
            let mut diagnostic = RuleDiagnostic::new(rule_category!(), range, format!("{err}"));

            for (range, err) in iter {
                diagnostic = diagnostic.detail(range, format!("{err}"));
            }

            Some(diagnostic)
        } else {
            None
        }
    }
}
