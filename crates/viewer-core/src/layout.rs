//! Automatic arrangement of a Diagram's nodes as a layered graph.

use rust_sugiyama::configure::Config;

use crate::scene::Rect;

const NODE_SPACING: f64 = 40.0;
const COMPONENT_GAP: f32 = 40.0;
/// Groups wrap onto a new row past this width, unless one group is wider.
const MAX_ROW_WIDTH: f32 = 900.0;

/// Places nodes of the given sizes so that every edge `(upper, lower)` runs
/// from a higher layer down to a lower one. Unconnected groups of nodes are
/// laid out in rows, in order of their first node. Returned rects are
/// indexed like `sizes`, with the top-left of the whole arrangement at the
/// origin.
pub fn layered(sizes: &[(f32, f32)], edges: &[(usize, usize)]) -> Vec<Rect> {
    if sizes.is_empty() {
        return Vec::new();
    }
    let vertices: Vec<(u32, (f64, f64))> = sizes
        .iter()
        .enumerate()
        .map(|(i, &(w, h))| (i as u32, (w as f64, h as f64)))
        .collect();
    let edges: Vec<(u32, u32)> = edges.iter().map(|&(a, b)| (a as u32, b as u32)).collect();
    let config = Config {
        vertex_spacing: NODE_SPACING,
        ..Config::default()
    };

    let mut components = rust_sugiyama::from_vertices_and_edges(&vertices, &edges, &config);
    for (positions, _, _) in &mut components {
        positions.sort_by_key(|&(id, _)| id);
    }
    components.sort_by_key(|(positions, _, _)| positions.first().map(|&(id, _)| id));

    let mut rects = vec![
        Rect {
            x: 0.0,
            y: 0.0,
            width: 0.0,
            height: 0.0
        };
        sizes.len()
    ];
    let (mut offset_x, mut offset_y, mut row_bottom) = (0.0_f32, 0.0_f32, 0.0_f32);
    for (positions, _, _) in components {
        // The library gives node centers; convert to top-left corners and
        // shift the component so its bounding box starts at `offset_x`.
        let placed: Vec<(usize, Rect)> = positions
            .iter()
            .map(|&(id, (cx, cy))| {
                let (width, height) = sizes[id];
                let rect = Rect {
                    x: cx.round() as f32 - width / 2.0,
                    y: cy.round() as f32 - height / 2.0,
                    width,
                    height,
                };
                (id, rect)
            })
            .collect();
        let min_x = placed
            .iter()
            .map(|(_, r)| r.x)
            .fold(f32::INFINITY, f32::min);
        let min_y = placed
            .iter()
            .map(|(_, r)| r.y)
            .fold(f32::INFINITY, f32::min);
        let width = placed
            .iter()
            .map(|(_, r)| r.right() - min_x)
            .fold(0.0, f32::max);
        if offset_x > 0.0 && offset_x + width > MAX_ROW_WIDTH {
            offset_x = 0.0;
            offset_y = row_bottom + COMPONENT_GAP;
        }
        for (id, rect) in placed {
            let rect = Rect {
                x: (rect.x - min_x + offset_x).round(),
                y: (rect.y - min_y + offset_y).round(),
                ..rect
            };
            row_bottom = row_bottom.max(rect.bottom());
            rects[id] = rect;
        }
        offset_x += width.round() + COMPONENT_GAP;
    }
    rects
}
