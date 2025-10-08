use std::cell::LazyCell;

use taffy::Style;
use tracing::error;

use crate::render::renderer::{AppState, NodeContext};

const POSSIBLE_PARAMETERS: LazyCell<Vec<StyleParameter>> = LazyCell::new(|| {
    let mut out = vec![
        StyleParameter::new("bg", StyleArgument::Color),
        StyleParameter::new("hover:bg", StyleArgument::Color),
        StyleParameter::new("rounded", StyleArgument::Size),
        StyleParameter::new("border", StyleArgument::Size),
        StyleParameter::new("border", StyleArgument::Color),
        StyleParameter::new("text", StyleArgument::Size),
        StyleParameter::new("text", StyleArgument::Color),
        StyleParameter::new("translate-x", StyleArgument::Length),
        StyleParameter::new("translate-y", StyleArgument::Length),
        StyleParameter::new("m", StyleArgument::Size),
        StyleParameter::new("mx", StyleArgument::Size),
        StyleParameter::new("my", StyleArgument::Size),
        StyleParameter::new("ml", StyleArgument::Size),
        StyleParameter::new("mr", StyleArgument::Size),
        StyleParameter::new("mt", StyleArgument::Size),
        StyleParameter::new("mb", StyleArgument::Size),
        StyleParameter::new("p", StyleArgument::Size),
        StyleParameter::new("px", StyleArgument::Size),
        StyleParameter::new("py", StyleArgument::Size),
        StyleParameter::new("pl", StyleArgument::Size),
        StyleParameter::new("pr", StyleArgument::Size),
        StyleParameter::new("pt", StyleArgument::Size),
        StyleParameter::new("pb", StyleArgument::Size),
        StyleParameter::new("flex-row", StyleArgument::None),
        StyleParameter::new("flex-col", StyleArgument::None),
        StyleParameter::new("flex-nowrap", StyleArgument::None),
        StyleParameter::new("flex-wrap", StyleArgument::None),
        StyleParameter::new("flex-wrap-reverse", StyleArgument::None),
        StyleParameter::new("grow", StyleArgument::None),
        StyleParameter::new("grow", StyleArgument::Length),
        StyleParameter::new("shrink", StyleArgument::None),
        StyleParameter::new("shrink", StyleArgument::Length),
        StyleParameter::new("gap", StyleArgument::Size),
        StyleParameter::new("items-start", StyleArgument::None),
        StyleParameter::new("items-end", StyleArgument::None),
        StyleParameter::new("items-end-safe", StyleArgument::None),
        StyleParameter::new("items-center", StyleArgument::None),
        StyleParameter::new("items-center-safe", StyleArgument::None),
        StyleParameter::new("items-baseline", StyleArgument::None),
        StyleParameter::new("items-baseline-last", StyleArgument::None),
        StyleParameter::new("items-stretch", StyleArgument::None),
        StyleParameter::new("self-start", StyleArgument::None),
        StyleParameter::new("self-end", StyleArgument::None),
        StyleParameter::new("self-end-safe", StyleArgument::None),
        StyleParameter::new("self-center", StyleArgument::None),
        StyleParameter::new("self-center-safe", StyleArgument::None),
        StyleParameter::new("self-baseline", StyleArgument::None),
        StyleParameter::new("self-baseline-last", StyleArgument::None),
        StyleParameter::new("self-stretch", StyleArgument::None),
        StyleParameter::new("justify-items-start", StyleArgument::None),
        StyleParameter::new("justify-items-end", StyleArgument::None),
        StyleParameter::new("justify-items-end-safe", StyleArgument::None),
        StyleParameter::new("justify-items-center", StyleArgument::None),
        StyleParameter::new("justify-items-center-safe", StyleArgument::None),
        StyleParameter::new("justify-items-stretch", StyleArgument::None),
        StyleParameter::new("justify-items-normal", StyleArgument::None),
        StyleParameter::new("justify-self-start", StyleArgument::None),
        StyleParameter::new("justify-self-end", StyleArgument::None),
        StyleParameter::new("justify-self-end-safe", StyleArgument::None),
        StyleParameter::new("justify-self-center", StyleArgument::None),
        StyleParameter::new("justify-self-center-safe", StyleArgument::None),
        StyleParameter::new("justify-self-stretch", StyleArgument::None),
    ];
    out.sort_by(|a, b| a.prefix.len().cmp(&b.prefix.len()));
    out
});

enum StyleArgument {
    /// Pixels, percent or named sizes (sm, md, lg, etc.)
    Size,
    /// Hexadecimal colors or named colors (sky-500, blue-200, etc.)
    Color,
    /// Pixels or percent. (Or just pixels???)
    Length,
    /// No argument. Used for keyword-like style parameters like flex-row, grow or none
    None,
}

struct StyleParameter {
    prefix: &'static str,
    argument: StyleArgument,
}

impl TryFrom<&str> for StyleParameter {
    type Error = String;

    fn try_from(value: &str) -> Result<Self, Self::Error> {
        todo!()
    }
}

impl StyleParameter {
    pub const fn new(prefix: &'static str, argument: StyleArgument) -> Self {
        Self { prefix, argument }
    }
}

pub fn parse_style<T>(style_str: &str) -> (Style, NodeContext<T>)
where
    T: AppState + Default,
{
    let mut style = Style::DEFAULT;
    let mut ctx = NodeContext::default();

    for param in style_str.split(" ") {
        for possible in &*POSSIBLE_PARAMETERS {
            if param.starts_with(possible.prefix) {
                let mut argument = &param[possible.prefix.len()..];
                if !argument.is_empty() {
                    argument = &argument[1..];
                }
                match possible.prefix {
                    "bg" => {}
                    "hover:bg" => {}
                    "rounded" => {}
                    "border" => {}
                    "text" => {}
                    "translate-x" => {}
                    "translate-y" => {}
                    "m" => {}
                    "mx" => {}
                    "my" => {}
                    "ml" => {}
                    "mr" => {}
                    "mt" => {}
                    "mb" => {}
                    "p" => {}
                    "px" => {}
                    "py" => {}
                    "pl" => {}
                    "pr" => {}
                    "pt" => {}
                    "pb" => {}
                    "flex-row" => {}
                    "flex-col" => {}
                    "flex-nowrap" => {}
                    "flex-wrap" => {}
                    "flex-wrap-reverse" => {}
                    "grow" => {}
                    "grow" => {}
                    "shrink" => {}
                    "shrink" => {}
                    "gap" => {}
                    "items-start" => {}
                    "items-end" => {}
                    "items-end-safe" => {}
                    "items-center" => {}
                    "items-center-safe" => {}
                    "items-baseline" => {}
                    "items-baseline-last" => {}
                    "items-stretch" => {}
                    "self-start" => {}
                    "self-end" => {}
                    "self-end-safe" => {}
                    "self-center" => {}
                    "self-center-safe" => {}
                    "self-baseline" => {}
                    "self-baseline-last" => {}
                    "self-stretch" => {}
                    "justify-items-start" => {}
                    "justify-items-end" => {}
                    "justify-items-end-safe" => {}
                    "justify-items-center" => {}
                    "justify-items-center-safe" => {}
                    "justify-items-stretch" => {}
                    "justify-items-normal" => {}
                    "justify-self-start" => {}
                    "justify-self-end" => {}
                    "justify-self-end-safe" => {}
                    "justify-self-center" => {}
                    "justify-self-center-safe" => {}
                    "justify-self-stretch" => {}
                    unknown => {
                        error!("Unknown style parameter {}", unknown);
                    }
                }
            }
        }
        break;
    }

    (style, ctx)
}
