//! Automatic arrangement of a Diagram's nodes as a layered graph.

use std::collections::{BTreeMap, HashMap, HashSet};

use rust_sugiyama::configure::Config;

use crate::scene::Rect;

const NODE_SPACING: f64 = 40.0;
const COMPONENT_GAP: f32 = 40.0;
/// Groups wrap onto a new row past this width, unless one group is wider.
const MAX_ROW_WIDTH: f32 = 900.0;

/// A node's children in a tree, in the order they are drawn left to
/// right. Its parent is `None` where it is placed by hand.
pub struct Family {
    pub parent: Option<usize>,
    pub children: Vec<usize>,
}

/// Places nodes of the given sizes so that every edge `(upper, lower)` runs
/// from a higher layer down to a lower one. Unconnected groups of nodes are
/// laid out in rows, in order of their first node. The children of each
/// of `families` that share a layer keep its order left to right, under
/// their parent. Returned rects are indexed like `sizes`, with the
/// top-left of the whole arrangement at the origin.
pub fn layered(sizes: &[(f32, f32)], edges: &[(usize, usize)], families: &[Family]) -> Vec<Rect> {
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
        keep_sibling_order(positions, sizes, families);
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

/// Lays each family's children out side by side in its order, centred
/// under their parent, top down, then moves nodes apart rightwards where a
/// layer overlaps. A node moves with all it is a parent of, so subtrees
/// stay under their parents.
fn keep_sibling_order(
    positions: &mut [(usize, (f64, f64))],
    sizes: &[(f32, f32)],
    families: &[Family],
) {
    let index: HashMap<usize, usize> = positions
        .iter()
        .enumerate()
        .map(|(i, &(id, _))| (id, i))
        .collect();
    let layer_of = |positions: &[(usize, (f64, f64))], i: usize| positions[i].1.1.round() as i64;
    let half = |positions: &[(usize, (f64, f64))], i: usize| sizes[positions[i].0].0 as f64 / 2.0;
    // Each node's children, by position, for moving subtrees.
    let mut below: HashMap<usize, Vec<usize>> = HashMap::new();
    for family in families {
        if let Some(&parent) = family.parent.as_ref().and_then(|p| index.get(p)) {
            below
                .entry(parent)
                .or_default()
                .extend(family.children.iter().filter_map(|c| index.get(c)));
        }
    }
    let shift = |positions: &mut [(usize, (f64, f64))], from: usize, dx: f64| {
        let mut moved = HashSet::new();
        let mut next = vec![from];
        while let Some(i) = next.pop() {
            if moved.insert(i) {
                positions[i].1.0 += dx;
                next.extend(below.get(&i).into_iter().flatten());
            }
        }
    };
    let mut families: Vec<(Option<usize>, Vec<usize>)> = families
        .iter()
        .map(|f| {
            let parent = f.parent.as_ref().and_then(|p| index.get(p)).copied();
            (
                parent,
                f.children
                    .iter()
                    .filter_map(|c| index.get(c))
                    .copied()
                    .collect(),
            )
        })
        .collect();
    families.sort_by_key(|(parent, _)| parent.map(|p| layer_of(positions, p)));
    for (parent, children) in &families {
        let mut layers: BTreeMap<i64, Vec<usize>> = BTreeMap::new();
        for &i in children {
            layers.entry(layer_of(positions, i)).or_default().push(i);
        }
        for (layer, members) in layers {
            let width: f64 = members
                .iter()
                .map(|&i| 2.0 * half(positions, i))
                .sum::<f64>()
                + NODE_SPACING * (members.len() - 1) as f64;
            let centre = match parent {
                Some(p) if layer_of(positions, *p) < layer => positions[*p].1.0,
                _ => {
                    let left = members
                        .iter()
                        .map(|&i| positions[i].1.0 - half(positions, i))
                        .fold(f64::INFINITY, f64::min);
                    left + width / 2.0
                }
            };
            let mut left = centre - width / 2.0;
            for i in members {
                let target = left + half(positions, i);
                left = target + half(positions, i) + NODE_SPACING;
                let dx = target - positions[i].1.0;
                shift(positions, i, dx);
            }
        }
    }
    let mut layers: BTreeMap<i64, Vec<usize>> = BTreeMap::new();
    for i in 0..positions.len() {
        layers.entry(layer_of(positions, i)).or_default().push(i);
    }
    for mut layer in layers.into_values() {
        layer.sort_by(|&a, &b| positions[a].1.0.total_cmp(&positions[b].1.0));
        let mut right = f64::NEG_INFINITY;
        for i in layer {
            let needed = right + NODE_SPACING + half(positions, i) - positions[i].1.0;
            if needed > 0.0 {
                shift(positions, i, needed);
            }
            right = positions[i].1.0 + half(positions, i);
        }
    }
}
