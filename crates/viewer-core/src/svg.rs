//! Serialises a scene as SVG. Output is deterministic: the same scene always
//! produces byte-identical text.

use std::fmt::Write;

use crate::scene::{Fill, Point, Scene, Shape, TextAnchor};

const STROKE: &str = "#000000";
const BACKGROUND: &str = "#ffffff";
const FONT_FAMILY: &str = "sans-serif";

pub fn render(scene: &Scene) -> String {
    let mut out = String::new();
    let (x, y, w, h) = (num(scene.x), num(scene.y), num(scene.width), num(scene.height));
    writeln!(
        out,
        r#"<svg xmlns="http://www.w3.org/2000/svg" width="{w}" height="{h}" viewBox="{x} {y} {w} {h}" font-family="{FONT_FAMILY}">"#
    )
    .unwrap();
    writeln!(out, r#"<rect x="{x}" y="{y}" width="{w}" height="{h}" fill="{BACKGROUND}"/>"#).unwrap();
    for item in &scene.items {
        shape(&mut out, &item.shape);
    }
    out.push_str("</svg>\n");
    out
}

fn shape(out: &mut String, shape: &Shape) {
    match shape {
        Shape::Rect { rect, fill, corner_radius } => writeln!(
            out,
            r#"<rect x="{}" y="{}" width="{}" height="{}"{} fill="{}" stroke="{STROKE}"/>"#,
            num(rect.x),
            num(rect.y),
            num(rect.width),
            num(rect.height),
            if *corner_radius > 0.0 { format!(r#" rx="{}""#, num(*corner_radius)) } else { String::new() },
            fill_color(*fill),
        ),
        Shape::Polyline { points, dashed } => writeln!(
            out,
            r#"<polyline points="{}" fill="none" stroke="{STROKE}"{}/>"#,
            point_list(points),
            if *dashed { r#" stroke-dasharray="6 4""# } else { "" },
        ),
        Shape::Polygon { points, fill } => writeln!(
            out,
            r#"<polygon points="{}" fill="{}" stroke="{STROKE}"/>"#,
            point_list(points),
            fill_color(*fill),
        ),
        Shape::Circle { center, radius, fill } => writeln!(
            out,
            r#"<circle cx="{}" cy="{}" r="{}" fill="{}"/>"#,
            num(center.x),
            num(center.y),
            num(*radius),
            fill_color(*fill),
        ),
        Shape::Text { position, text, anchor, font } => writeln!(
            out,
            r#"<text x="{}" y="{}" text-anchor="{}" font-size="{}"{}{}>{}</text>"#,
            num(position.x),
            num(position.y),
            match anchor {
                TextAnchor::Start => "start",
                TextAnchor::Middle => "middle",
                TextAnchor::End => "end",
            },
            num(font.size),
            if font.bold { r#" font-weight="bold""# } else { "" },
            if font.italic { r#" font-style="italic""# } else { "" },
            escape(text),
        ),
    }
    .unwrap();
}

fn fill_color(fill: Fill) -> &'static str {
    match fill {
        Fill::None => "none",
        Fill::Background => BACKGROUND,
        Fill::Stroke => STROKE,
    }
}

fn point_list(points: &[Point]) -> String {
    points
        .iter()
        .map(|p| format!("{},{}", num(p.x), num(p.y)))
        .collect::<Vec<_>>()
        .join(" ")
}

/// Fixed two-decimal formatting with trailing zeros trimmed, so tiny
/// floating-point differences never reach the output.
fn num(value: f32) -> String {
    let text = format!("{:.2}", value);
    let text = text.trim_end_matches('0').trim_end_matches('.');
    if text == "-0" { "0".to_owned() } else { text.to_owned() }
}

fn escape(text: &str) -> String {
    text.replace('&', "&amp;")
        .replace('<', "&lt;")
        .replace('>', "&gt;")
        .replace('"', "&quot;")
}
