use std::{
    cell::LazyCell,
    sync::{LazyLock, RwLock},
};

use taffy::{Dimension, FlexDirection, Style};
use tracing::error;

use crate::render::{
    BorderRadius, Color,
    renderer::{AppState, NodeContext, flags},
};

const POSSIBLE_PARAMETERS: LazyCell<Vec<&str>> = LazyCell::new(|| {
    let mut out = vec![
        "bg",
        "hover:bg",
        "rounded",
        "border",
        "border",
        "text",
        "text",
        "translate-x",
        "translate-y",
        "m",
        "mx",
        "my",
        "ml",
        "mr",
        "mt",
        "mb",
        "p",
        "px",
        "py",
        "pl",
        "pr",
        "pt",
        "pb",
        "flex-row",
        "flex-col",
        "flex-nowrap",
        "flex-wrap",
        "flex-wrap-reverse",
        "grow",
        "grow",
        "shrink",
        "shrink",
        "gap",
        "items-start",
        "items-end",
        "items-end-safe",
        "items-center",
        "items-center-safe",
        "items-baseline",
        "items-baseline-last",
        "items-stretch",
        "self-start",
        "self-end",
        "self-end-safe",
        "self-center",
        "self-center-safe",
        "self-baseline",
        "self-baseline-last",
        "self-stretch",
        "justify-items-start",
        "justify-items-end",
        "justify-items-end-safe",
        "justify-items-center",
        "justify-items-center-safe",
        "justify-items-stretch",
        "justify-self-start",
        "justify-self-end",
        "justify-self-end-safe",
        "justify-self-center",
        "justify-self-center-safe",
        "justify-self-stretch",
        "opacity",
        "w",
        "h",
        "max-w",
        "max-h",
        "min-w",
        "min-h",
        "overflow-clip",
        "scroll-bar",
        "scroll-content",
    ];
    out.sort_by(|a, b| b.len().cmp(&a.len()));
    out
});

fn hex(h: &str) -> Color {
    let h = h.trim_start_matches('#');
    let (r, g, b) = match h.len() {
        6 => (
            u8::from_str_radix(&h[0..2], 16).unwrap(),
            u8::from_str_radix(&h[2..4], 16).unwrap(),
            u8::from_str_radix(&h[4..6], 16).unwrap(),
        ),
        _ => panic!("bad hex"),
    };
    Color {
        r: r as f32 / 255.0,
        g: g as f32 / 255.0,
        b: b as f32 / 255.0,
        a: 1.0,
    }
}

pub static TAILWIND_COLORS: LazyLock<RwLock<Vec<(String, Color)>>> = LazyLock::new(|| {
    RwLock::new(vec![
        // slate
        (String::from("slate-50"), hex("#f8fafc")),
        (String::from("slate-100"), hex("#f1f5f9")),
        (String::from("slate-200"), hex("#e2e8f0")),
        (String::from("slate-300"), hex("#cbd5e1")),
        (String::from("slate-400"), hex("#94a3b8")),
        (String::from("slate-500"), hex("#64748b")),
        (String::from("slate-600"), hex("#475569")),
        (String::from("slate-700"), hex("#334155")),
        (String::from("slate-800"), hex("#1f2937")),
        (String::from("slate-900"), hex("#0f172a")),
        (String::from("slate-950"), hex("#020617")),
        // gray
        (String::from("gray-50"), hex("#f9fafb")),
        (String::from("gray-100"), hex("#f3f4f6")),
        (String::from("gray-200"), hex("#e5e7eb")),
        (String::from("gray-300"), hex("#d1d5db")),
        (String::from("gray-400"), hex("#9ca3af")),
        (String::from("gray-500"), hex("#6b7280")),
        (String::from("gray-600"), hex("#4b5563")),
        (String::from("gray-700"), hex("#374151")),
        (String::from("gray-800"), hex("#1f2937")),
        (String::from("gray-900"), hex("#111827")),
        (String::from("gray-950"), hex("#030712")),
        // zinc
        (String::from("zinc-50"), hex("#fafafa")),
        (String::from("zinc-100"), hex("#f4f4f5")),
        (String::from("zinc-200"), hex("#e4e4e7")),
        (String::from("zinc-300"), hex("#d4d4d8")),
        (String::from("zinc-400"), hex("#a1a1aa")),
        (String::from("zinc-500"), hex("#71717a")),
        (String::from("zinc-600"), hex("#52525b")),
        (String::from("zinc-700"), hex("#3f3f46")),
        (String::from("zinc-800"), hex("#27272a")),
        (String::from("zinc-900"), hex("#18181b")),
        (String::from("zinc-950"), hex("#09090b")),
        // neutral
        (String::from("neutral-50"), hex("#fafafa")),
        (String::from("neutral-100"), hex("#f5f5f5")),
        (String::from("neutral-200"), hex("#e5e5e5")),
        (String::from("neutral-300"), hex("#d4d4d4")),
        (String::from("neutral-400"), hex("#a3a3a3")),
        (String::from("neutral-500"), hex("#737373")),
        (String::from("neutral-600"), hex("#525252")),
        (String::from("neutral-700"), hex("#404040")),
        (String::from("neutral-800"), hex("#262626")),
        (String::from("neutral-900"), hex("#171717")),
        (String::from("neutral-950"), hex("#0a0a0a")),
        // stone
        (String::from("stone-50"), hex("#fafaf9")),
        (String::from("stone-100"), hex("#f5f5f4")),
        (String::from("stone-200"), hex("#e7e5e4")),
        (String::from("stone-300"), hex("#d6d3d1")),
        (String::from("stone-400"), hex("#a8a29e")),
        (String::from("stone-500"), hex("#78716c")),
        (String::from("stone-600"), hex("#57534e")),
        (String::from("stone-700"), hex("#44403c")),
        (String::from("stone-800"), hex("#292524")),
        (String::from("stone-900"), hex("#1c1917")),
        (String::from("stone-950"), hex("#0c0a09")),
        // red
        (String::from("red-50"), hex("#fef2f2")),
        (String::from("red-100"), hex("#fee2e2")),
        (String::from("red-200"), hex("#fecaca")),
        (String::from("red-300"), hex("#fca5a5")),
        (String::from("red-400"), hex("#f87171")),
        (String::from("red-500"), hex("#ef4444")),
        (String::from("red-600"), hex("#dc2626")),
        (String::from("red-700"), hex("#b91c1c")),
        (String::from("red-800"), hex("#991b1b")),
        (String::from("red-900"), hex("#7f1d1d")),
        (String::from("red-950"), hex("#450a0a")),
        // orange
        (String::from("orange-50"), hex("#fff7ed")),
        (String::from("orange-100"), hex("#ffedd5")),
        (String::from("orange-200"), hex("#fed7aa")),
        (String::from("orange-300"), hex("#fdba74")),
        (String::from("orange-400"), hex("#fb923c")),
        (String::from("orange-500"), hex("#f97316")),
        (String::from("orange-600"), hex("#ea580c")),
        (String::from("orange-700"), hex("#c2410c")),
        (String::from("orange-800"), hex("#9a3412")),
        (String::from("orange-900"), hex("#7c2d12")),
        (String::from("orange-950"), hex("#431407")),
        // amber
        (String::from("amber-50"), hex("#fffbeb")),
        (String::from("amber-100"), hex("#fef3c7")),
        (String::from("amber-200"), hex("#fde68a")),
        (String::from("amber-300"), hex("#fcd34d")),
        (String::from("amber-400"), hex("#fbbf24")),
        (String::from("amber-500"), hex("#f59e0b")),
        (String::from("amber-600"), hex("#d97706")),
        (String::from("amber-700"), hex("#b45309")),
        (String::from("amber-800"), hex("#92400e")),
        (String::from("amber-900"), hex("#78350f")),
        (String::from("amber-950"), hex("#451a03")),
        // yellow
        (String::from("yellow-50"), hex("#fefce8")),
        (String::from("yellow-100"), hex("#fef9c3")),
        (String::from("yellow-200"), hex("#fef08a")),
        (String::from("yellow-300"), hex("#fde047")),
        (String::from("yellow-400"), hex("#facc15")),
        (String::from("yellow-500"), hex("#eab308")),
        (String::from("yellow-600"), hex("#ca8a04")),
        (String::from("yellow-700"), hex("#a16207")),
        (String::from("yellow-800"), hex("#854d0e")),
        (String::from("yellow-900"), hex("#713f12")),
        (String::from("yellow-950"), hex("#422006")),
        // lime
        (String::from("lime-50"), hex("#f7fee7")),
        (String::from("lime-100"), hex("#ecfccb")),
        (String::from("lime-200"), hex("#d9f99d")),
        (String::from("lime-300"), hex("#bef264")),
        (String::from("lime-400"), hex("#a3e635")),
        (String::from("lime-500"), hex("#84cc16")),
        (String::from("lime-600"), hex("#65a30d")),
        (String::from("lime-700"), hex("#4d7c0f")),
        (String::from("lime-800"), hex("#3f6212")),
        (String::from("lime-900"), hex("#365314")),
        (String::from("lime-950"), hex("#1a2e05")),
        // green
        (String::from("green-50"), hex("#f0fdf4")),
        (String::from("green-100"), hex("#dcfce7")),
        (String::from("green-200"), hex("#bbf7d0")),
        (String::from("green-300"), hex("#86efac")),
        (String::from("green-400"), hex("#4ade80")),
        (String::from("green-500"), hex("#22c55e")),
        (String::from("green-600"), hex("#16a34a")),
        (String::from("green-700"), hex("#15803d")),
        (String::from("green-800"), hex("#166534")),
        (String::from("green-900"), hex("#14532d")),
        (String::from("green-950"), hex("#052e16")),
        // emerald
        (String::from("emerald-50"), hex("#ecfdf5")),
        (String::from("emerald-100"), hex("#d1fae5")),
        (String::from("emerald-200"), hex("#a7f3d0")),
        (String::from("emerald-300"), hex("#6ee7b7")),
        (String::from("emerald-400"), hex("#34d399")),
        (String::from("emerald-500"), hex("#10b981")),
        (String::from("emerald-600"), hex("#059669")),
        (String::from("emerald-700"), hex("#047857")),
        (String::from("emerald-800"), hex("#065f46")),
        (String::from("emerald-900"), hex("#064e3b")),
        (String::from("emerald-950"), hex("#022c22")),
        // teal
        (String::from("teal-50"), hex("#f0fdfa")),
        (String::from("teal-100"), hex("#ccfbf1")),
        (String::from("teal-200"), hex("#99f6e4")),
        (String::from("teal-300"), hex("#5eead4")),
        (String::from("teal-400"), hex("#2dd4bf")),
        (String::from("teal-500"), hex("#14b8a6")),
        (String::from("teal-600"), hex("#0d9488")),
        (String::from("teal-700"), hex("#0f766e")),
        (String::from("teal-800"), hex("#115e59")),
        (String::from("teal-900"), hex("#134e4a")),
        (String::from("teal-950"), hex("#042f2e")),
        // cyan
        (String::from("cyan-50"), hex("#ecfeff")),
        (String::from("cyan-100"), hex("#cffafe")),
        (String::from("cyan-200"), hex("#a5f3fc")),
        (String::from("cyan-300"), hex("#67e8f9")),
        (String::from("cyan-400"), hex("#22d3ee")),
        (String::from("cyan-500"), hex("#06b6d4")),
        (String::from("cyan-600"), hex("#0891b2")),
        (String::from("cyan-700"), hex("#0e7490")),
        (String::from("cyan-800"), hex("#155e75")),
        (String::from("cyan-900"), hex("#164e63")),
        (String::from("cyan-950"), hex("#083344")),
        // sky
        (String::from("sky-50"), hex("#f0f9ff")),
        (String::from("sky-100"), hex("#e0f2fe")),
        (String::from("sky-200"), hex("#bae6fd")),
        (String::from("sky-300"), hex("#7dd3fc")),
        (String::from("sky-400"), hex("#38bdf8")),
        (String::from("sky-500"), hex("#0ea5e9")),
        (String::from("sky-600"), hex("#0284c7")),
        (String::from("sky-700"), hex("#0369a1")),
        (String::from("sky-800"), hex("#075985")),
        (String::from("sky-900"), hex("#0c4a6e")),
        (String::from("sky-950"), hex("#082f49")),
        // blue
        (String::from("blue-50"), hex("#eff6ff")),
        (String::from("blue-100"), hex("#dbeafe")),
        (String::from("blue-200"), hex("#bfdbfe")),
        (String::from("blue-300"), hex("#93c5fd")),
        (String::from("blue-400"), hex("#60a5fa")),
        (String::from("blue-500"), hex("#3b82f6")),
        (String::from("blue-600"), hex("#2563eb")),
        (String::from("blue-700"), hex("#1d4ed8")),
        (String::from("blue-800"), hex("#1e40af")),
        (String::from("blue-900"), hex("#1e3a8a")),
        (String::from("blue-950"), hex("#172554")),
        // indigo
        (String::from("indigo-50"), hex("#eef2ff")),
        (String::from("indigo-100"), hex("#e0e7ff")),
        (String::from("indigo-200"), hex("#c7d2fe")),
        (String::from("indigo-300"), hex("#a5b4fc")),
        (String::from("indigo-400"), hex("#818cf8")),
        (String::from("indigo-500"), hex("#6366f1")),
        (String::from("indigo-600"), hex("#4f46e5")),
        (String::from("indigo-700"), hex("#4338ca")),
        (String::from("indigo-800"), hex("#3730a3")),
        (String::from("indigo-900"), hex("#312e81")),
        (String::from("indigo-950"), hex("#1e1b4b")),
        // violet
        (String::from("violet-50"), hex("#f5f3ff")),
        (String::from("violet-100"), hex("#ede9fe")),
        (String::from("violet-200"), hex("#ddd6fe")),
        (String::from("violet-300"), hex("#c4b5fd")),
        (String::from("violet-400"), hex("#a78bfa")),
        (String::from("violet-500"), hex("#8b5cf6")),
        (String::from("violet-600"), hex("#7c3aed")),
        (String::from("violet-700"), hex("#6d28d9")),
        (String::from("violet-800"), hex("#5b21b6")),
        (String::from("violet-900"), hex("#4c1d95")),
        (String::from("violet-950"), hex("#2e1065")),
        // purple
        (String::from("purple-50"), hex("#faf5ff")),
        (String::from("purple-100"), hex("#f3e8ff")),
        (String::from("purple-200"), hex("#e9d5ff")),
        (String::from("purple-300"), hex("#d8b4fe")),
        (String::from("purple-400"), hex("#c084fc")),
        (String::from("purple-500"), hex("#a855f7")),
        (String::from("purple-600"), hex("#9333ea")),
        (String::from("purple-700"), hex("#7e22ce")),
        (String::from("purple-800"), hex("#6b21a8")),
        (String::from("purple-900"), hex("#581c87")),
        (String::from("purple-950"), hex("#3b0764")),
        // fuchsia
        (String::from("fuchsia-50"), hex("#fdf4ff")),
        (String::from("fuchsia-100"), hex("#fae8ff")),
        (String::from("fuchsia-200"), hex("#f5d0fe")),
        (String::from("fuchsia-300"), hex("#f0abfc")),
        (String::from("fuchsia-400"), hex("#e879f9")),
        (String::from("fuchsia-500"), hex("#d946ef")),
        (String::from("fuchsia-600"), hex("#c026d3")),
        (String::from("fuchsia-700"), hex("#a21caf")),
        (String::from("fuchsia-800"), hex("#86198f")),
        (String::from("fuchsia-900"), hex("#701a75")),
        (String::from("fuchsia-950"), hex("#4a044e")),
        // pink
        (String::from("pink-50"), hex("#fdf2f8")),
        (String::from("pink-100"), hex("#fce7f3")),
        (String::from("pink-200"), hex("#fbcfe8")),
        (String::from("pink-300"), hex("#f9a8d4")),
        (String::from("pink-400"), hex("#f472b6")),
        (String::from("pink-500"), hex("#ec4899")),
        (String::from("pink-600"), hex("#db2777")),
        (String::from("pink-700"), hex("#be185d")),
        (String::from("pink-800"), hex("#9d174d")),
        (String::from("pink-900"), hex("#831843")),
        (String::from("pink-950"), hex("#500724")),
        // rose
        (String::from("rose-50"), hex("#fff1f2")),
        (String::from("rose-100"), hex("#ffe4e6")),
        (String::from("rose-200"), hex("#fecdd3")),
        (String::from("rose-300"), hex("#fda4af")),
        (String::from("rose-400"), hex("#fb7185")),
        (String::from("rose-500"), hex("#f43f5e")),
        (String::from("rose-600"), hex("#e11d48")),
        (String::from("rose-700"), hex("#be123c")),
        (String::from("rose-800"), hex("#9f1239")),
        (String::from("rose-900"), hex("#881337")),
        (String::from("rose-950"), hex("#4c0519")),
        // black/white
        (String::from("black"), hex("#000000")),
        (String::from("white"), hex("#ffffff")),
    ])
});

pub fn add_tailwind_color(name: String, color: Color) {
    TAILWIND_COLORS.write().unwrap().push((name, color));
}

enum StyleArgument {
    /// Hexadecimal colors or named colors (sky-500, blue-200, etc.)
    Color(Color),
    /// A length in pixels
    Length(f32),
    /// A length in percent
    Percent(f32),
    /// Auto dimension
    Auto,
    /// No argument. Used for keyword-like style parameters like flex-row, grow or none
    None,
}

impl TryFrom<&str> for StyleArgument {
    type Error = anyhow::Error;

    fn try_from(value: &str) -> Result<Self, Self::Error> {
        if value.is_empty() {
            Ok(StyleArgument::None)
        } else if value == "auto" {
            Ok(StyleArgument::Auto)
        } else {
            if value.starts_with("#") && value.chars().all(|c| c.is_ascii_hexdigit()) {
                Ok(StyleArgument::Color(hex(value)))
            } else if let Some((_, color)) = TAILWIND_COLORS
                .read()
                .unwrap()
                .iter()
                .filter(|(color_code, _)| color_code == value)
                .next()
            {
                Ok(StyleArgument::Color(*color))
            } else if value.chars().all(|c| c.is_numeric()) {
                Ok(StyleArgument::Length(value.parse()?))
            } else if value == "full" {
                Ok(StyleArgument::Percent(1.0))
            } else if value == "1/2" {
                Ok(StyleArgument::Percent(0.5))
            } else if value == "1/3" {
                Ok(StyleArgument::Percent(1.0 / 3.0))
            } else if value == "1/4" {
                Ok(StyleArgument::Percent(0.25))
            } else {
                Err(anyhow::anyhow!("Invalid argument"))
            }
        }
    }
}

pub fn parse_style<T>(style_str: &str) -> (Style, NodeContext<T>)
where
    T: AppState + Default,
{
    let mut style = Style::DEFAULT;
    let mut ctx = NodeContext::default();

    if style_str.is_empty() {
        return (style, ctx);
    }

    for param in style_str.split(" ") {
        if param.is_empty() {
            continue;
        }
        let mut found = false;
        for possible in &*POSSIBLE_PARAMETERS {
            if param.starts_with(possible) {
                let mut argument = &param[possible.len()..];
                if !argument.is_empty() {
                    argument = &argument[1..];
                }
                if let Ok(argument) = StyleArgument::try_from(argument) {
                    match (*possible, argument) {
                        ("bg", StyleArgument::Color(color)) => {
                            ctx.bg_color = color;
                        }
                        ("hover:bg", StyleArgument::Color(color)) => {
                            ctx.flags |= flags::HOVER_BG;
                            ctx.bg_color_hover = color;
                        }
                        ("rounded", StyleArgument::Length(length)) => {
                            ctx.border.radius = BorderRadius::all(length);
                        }
                        ("border", StyleArgument::Length(length)) => {
                            ctx.border.thickness = length;
                        }
                        ("border", StyleArgument::Color(color)) => {
                            ctx.border.color = color;
                        }
                        ("text", StyleArgument::Length(length)) => {
                            ctx.text.font_size = length as u32;
                        }
                        ("text", StyleArgument::Color(color)) => {
                            ctx.text.color = color;
                        }
                        ("translate-x", StyleArgument::Length(length)) => {
                            ctx.offset.x = length;
                        }
                        ("translate-y", StyleArgument::Length(length)) => {
                            ctx.offset.y = length;
                        }
                        ("m", StyleArgument::Length(length)) => {
                            style.margin = taffy::Rect::length(length);
                        }
                        ("m", StyleArgument::Auto) => {
                            style.margin = taffy::Rect::auto();
                        }
                        ("mx", StyleArgument::Length(length)) => {
                            style.margin.left = taffy::prelude::length(length);
                            style.margin.right = taffy::prelude::length(length);
                        }
                        ("mx", StyleArgument::Auto) => {
                            style.margin.left = taffy::prelude::auto();
                            style.margin.right = taffy::prelude::auto();
                        }
                        ("my", StyleArgument::Length(length)) => {
                            style.margin.top = taffy::prelude::length(length);
                            style.margin.bottom = taffy::prelude::length(length);
                        }
                        ("my", StyleArgument::Auto) => {
                            style.margin.top = taffy::prelude::auto();
                            style.margin.bottom = taffy::prelude::auto();
                        }
                        ("ml", StyleArgument::Length(length)) => {
                            style.margin.left = taffy::prelude::length(length);
                        }
                        ("ml", StyleArgument::Auto) => {
                            style.margin.left = taffy::prelude::auto();
                        }
                        ("mr", StyleArgument::Length(length)) => {
                            style.margin.right = taffy::prelude::length(length);
                        }
                        ("mr", StyleArgument::Auto) => {
                            style.margin.right = taffy::prelude::auto();
                        }
                        ("mt", StyleArgument::Length(length)) => {
                            style.margin.top = taffy::prelude::length(length);
                        }
                        ("mt", StyleArgument::Auto) => {
                            style.margin.top = taffy::prelude::auto();
                        }
                        ("mb", StyleArgument::Length(length)) => {
                            style.margin.bottom = taffy::prelude::length(length);
                        }
                        ("mb", StyleArgument::Auto) => {
                            style.margin.bottom = taffy::prelude::auto();
                        }
                        ("p", StyleArgument::Length(length)) => {
                            style.padding = taffy::Rect::length(length);
                        }
                        ("px", StyleArgument::Length(length)) => {
                            style.padding.left = taffy::prelude::length(length);
                            style.padding.right = taffy::prelude::length(length);
                        }
                        ("py", StyleArgument::Length(length)) => {
                            style.padding.top = taffy::prelude::length(length);
                            style.padding.bottom = taffy::prelude::length(length);
                        }
                        ("pl", StyleArgument::Length(length)) => {
                            style.padding.left = taffy::prelude::length(length);
                        }
                        ("pr", StyleArgument::Length(length)) => {
                            style.padding.right = taffy::prelude::length(length);
                        }
                        ("pt", StyleArgument::Length(length)) => {
                            style.padding.top = taffy::prelude::length(length);
                        }
                        ("pb", StyleArgument::Length(length)) => {
                            style.padding.bottom = taffy::prelude::length(length);
                        }
                        ("flex-row", StyleArgument::None) => {
                            style.flex_direction = FlexDirection::Row;
                        }
                        ("flex-col", StyleArgument::None) => {
                            style.flex_direction = FlexDirection::Column;
                        }
                        ("flex-nowrap", StyleArgument::None) => {
                            style.flex_wrap = taffy::FlexWrap::NoWrap;
                        }
                        ("flex-wrap", StyleArgument::None) => {
                            style.flex_wrap = taffy::FlexWrap::Wrap;
                        }
                        ("flex-wrap-reverse", StyleArgument::None) => {
                            style.flex_wrap = taffy::FlexWrap::WrapReverse;
                        }
                        ("grow", StyleArgument::None) => {
                            style.flex_grow = 1.0;
                        }
                        ("grow", StyleArgument::Length(length)) => {
                            style.flex_grow = length;
                        }
                        ("shrink", StyleArgument::None) => {
                            style.flex_shrink = 1.0;
                        }
                        ("shrink", StyleArgument::Length(length)) => {
                            style.flex_shrink = length;
                        }
                        ("gap", StyleArgument::Length(length)) => {
                            style.gap = taffy::Size::length(length);
                        }
                        ("items-start", StyleArgument::None) => {
                            style.align_items = Some(taffy::AlignItems::Start);
                        }
                        ("items-end", StyleArgument::None) => {
                            style.align_items = Some(taffy::AlignItems::End);
                        }
                        ("items-end-safe", StyleArgument::None) => {
                            style.align_items = Some(taffy::AlignItems::End);
                        }
                        ("items-center", StyleArgument::None) => {
                            style.align_items = Some(taffy::AlignItems::Center);
                        }
                        ("items-center-safe", StyleArgument::None) => {
                            style.align_items = Some(taffy::AlignItems::Center);
                        }
                        ("items-baseline", StyleArgument::None) => {
                            style.align_items = Some(taffy::AlignItems::Baseline);
                        }
                        ("items-baseline-last", StyleArgument::None) => {
                            style.align_items = Some(taffy::AlignItems::Baseline);
                        }
                        ("items-stretch", StyleArgument::None) => {
                            style.align_items = Some(taffy::AlignItems::Stretch);
                        }
                        ("self-start", StyleArgument::None) => {
                            style.align_self = Some(taffy::AlignSelf::Start);
                        }
                        ("self-end", StyleArgument::None) => {
                            style.align_self = Some(taffy::AlignSelf::End);
                        }
                        ("self-end-safe", StyleArgument::None) => {
                            style.align_self = Some(taffy::AlignSelf::End);
                        }
                        ("self-center", StyleArgument::None) => {
                            style.align_self = Some(taffy::AlignSelf::Center);
                        }
                        ("self-center-safe", StyleArgument::None) => {
                            style.align_self = Some(taffy::AlignSelf::Center);
                        }
                        ("self-baseline", StyleArgument::None) => {
                            style.align_self = Some(taffy::AlignSelf::Baseline);
                        }
                        ("self-baseline-last", StyleArgument::None) => {
                            style.align_self = Some(taffy::AlignSelf::Baseline);
                        }
                        ("self-stretch", StyleArgument::None) => {
                            style.align_self = Some(taffy::AlignSelf::Stretch);
                        }
                        ("justify-items-start", StyleArgument::None) => {
                            style.justify_items = Some(taffy::JustifyItems::Start);
                        }
                        ("justify-items-end", StyleArgument::None) => {
                            style.justify_items = Some(taffy::JustifyItems::End);
                        }
                        ("justify-items-end-safe", StyleArgument::None) => {
                            style.justify_items = Some(taffy::JustifyItems::End);
                        }
                        ("justify-items-center", StyleArgument::None) => {
                            style.justify_items = Some(taffy::JustifyItems::Center);
                        }
                        ("justify-items-center-safe", StyleArgument::None) => {
                            style.justify_items = Some(taffy::JustifyItems::Center);
                        }
                        ("justify-items-stretch", StyleArgument::None) => {
                            style.justify_items = Some(taffy::JustifyItems::Stretch);
                        }
                        ("justify-self-start", StyleArgument::None) => {
                            style.justify_self = Some(taffy::JustifySelf::Start);
                        }
                        ("justify-self-end", StyleArgument::None) => {
                            style.justify_self = Some(taffy::JustifySelf::End);
                        }
                        ("justify-self-end-safe", StyleArgument::None) => {
                            style.justify_self = Some(taffy::JustifySelf::End);
                        }
                        ("justify-self-center", StyleArgument::None) => {
                            style.justify_self = Some(taffy::JustifySelf::Center);
                        }
                        ("justify-self-center-safe", StyleArgument::None) => {
                            style.justify_self = Some(taffy::JustifySelf::Center);
                        }
                        ("justify-self-stretch", StyleArgument::None) => {
                            style.justify_self = Some(taffy::JustifySelf::Stretch);
                        }
                        ("opacity", StyleArgument::Length(length)) => {
                            ctx.bg_color.a = length / 100.0;
                        }
                        ("w", StyleArgument::Length(length)) => {
                            style.size.width = Dimension::length(length);
                        }
                        ("w", StyleArgument::Percent(percent)) => {
                            style.size.width = Dimension::percent(percent);
                        }
                        ("w", StyleArgument::Auto) => {
                            style.size.width = Dimension::auto();
                        }
                        ("h", StyleArgument::Length(length)) => {
                            style.size.height = Dimension::length(length);
                        }
                        ("h", StyleArgument::Percent(percent)) => {
                            style.size.height = Dimension::percent(percent);
                        }
                        ("h", StyleArgument::Auto) => {
                            style.size.height = Dimension::auto();
                        }
                        ("max-w", StyleArgument::Length(length)) => {
                            style.max_size.width = Dimension::length(length);
                        }
                        ("max-w", StyleArgument::Percent(percent)) => {
                            style.max_size.width = Dimension::percent(percent);
                        }
                        ("max-w", StyleArgument::Auto) => {
                            style.max_size.width = Dimension::auto();
                        }
                        ("max-h", StyleArgument::Length(length)) => {
                            style.max_size.height = Dimension::length(length);
                        }
                        ("max-h", StyleArgument::Percent(percent)) => {
                            style.max_size.height = Dimension::percent(percent);
                        }
                        ("max-h", StyleArgument::Auto) => {
                            style.max_size.height = Dimension::auto();
                        }
                        ("min-w", StyleArgument::Length(length)) => {
                            style.min_size.width = Dimension::length(length);
                        }
                        ("min-w", StyleArgument::Percent(percent)) => {
                            style.min_size.width = Dimension::percent(percent);
                        }
                        ("min-w", StyleArgument::Auto) => {
                            style.min_size.width = Dimension::auto();
                        }
                        ("min-h", StyleArgument::Length(length)) => {
                            style.min_size.height = Dimension::length(length);
                        }
                        ("min-h", StyleArgument::Percent(percent)) => {
                            style.min_size.height = Dimension::percent(percent);
                        }
                        ("min-h", StyleArgument::Auto) => {
                            style.min_size.height = Dimension::auto();
                        }
                        ("overflow-clip", StyleArgument::None) => {
                            ctx.scissor = true;
                            style.overflow = taffy::Point {
                                x: taffy::Overflow::Hidden,
                                y: taffy::Overflow::Hidden,
                            };
                        }
                        ("scroll-bar", StyleArgument::None) => {
                            if ctx.flags & flags::SCROLL_CONTENT != 0 {
                                error!("An element can't be both scroll-bar and scroll-content");
                            }
                            ctx.flags |= flags::SCROLL_BAR;
                        }
                        ("scroll-content", StyleArgument::None) => {
                            if ctx.flags & flags::SCROLL_BAR != 0 {
                                error!("An element can't be both scroll-bar and scroll-content");
                            }
                            ctx.flags |= flags::SCROLL_CONTENT;
                        }
                        _ => {
                            error!("Unknown style argument-parameter combination {:?}", param);
                        }
                    }
                } else {
                    error!("Unknown style argument {}", argument);
                }
                found = true;
                break;
            }
        }
        if !found {
            error!("Unknown style {:?}", param);
        }
    }

    (style, ctx)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[derive(Default)]
    struct DummyState {}

    impl AppState for DummyState {
        fn generate_layout(
            &mut self,
            _: crate::geometry::Vector<f32>,
        ) -> Vec<crate::render::renderer::RenderLayout<Self>> {
            todo!()
        }
    }

    #[test]
    pub fn can_parse_basic_style() {
        let style_str = "rounded-8 bg-black";
        let (style, ctx) = parse_style::<DummyState>(style_str);

        assert!(
            ctx.border.radius.top_left == 8.0,
            "Border radius should be 8"
        );
        assert!(
            ctx.border.radius.top_right == 8.0,
            "Border radius should be 8"
        );
        assert!(
            ctx.border.radius.bottom_left == 8.0,
            "Border radius should be 8"
        );
        assert!(
            ctx.border.radius.bottom_right == 8.0,
            "Border radius should be 8"
        );
    }

    #[test]
    pub fn hover_bg_and_bg_parsing() {
        let style_str = "w-full bg-red-800 hover:bg-red-900 h-16 rounded-4";
        let (style, ctx) = parse_style::<DummyState>(style_str);
        assert!(
            (ctx.flags & flags::HOVER_BG) != 0,
            "Should have a hover bg color"
        );
        let red_900 = hex("#7f1d1d");
        assert!(
            ctx.bg_color_hover == red_900,
            "The hover color should be red-900"
        );
    }
}
