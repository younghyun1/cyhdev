//! Brick grid layout and colours, kept free of browser types so the rules run
//! under host unit tests.

const MARGIN_X: f64 = 40.0;
const TOP: f64 = 80.0;
const SPACING: f64 = 12.0;
const BRICK_HEIGHT: f64 = 28.0;
const MIN_USABLE_WIDTH: f64 = 200.0;

#[derive(Clone, Debug, PartialEq)]
pub(crate) struct Brick {
    pub(crate) x: f64,
    pub(crate) y: f64,
    pub(crate) w: f64,
    pub(crate) h: f64,
    pub(crate) alive: bool,
    pub(crate) hue: f64,
    pub(crate) palette: BrickPalette,
}

/// Fill and glow colours for one brick, formatted once per layout instead of
/// three `format!` calls per brick per frame.
#[derive(Clone, Debug, PartialEq)]
pub(crate) struct BrickPalette {
    pub(crate) top: String,
    pub(crate) bottom: String,
    pub(crate) glow: String,
}

impl BrickPalette {
    pub(crate) fn for_hue(hue: f64) -> Self {
        Self {
            top: format!("hsla({hue}, 88%, 68%, 0.95)"),
            bottom: format!("hsla({hue}, 80%, 46%, 0.95)"),
            glow: format!("hsla({hue}, 90%, 60%, 0.7)"),
        }
    }
}

/// Lays out a fresh level of `rows` by `cols` live bricks across a viewport
/// `width` CSS pixels wide, with hues shifting by row and column.
pub(crate) fn brick_grid(width: f64, rows: usize, cols: usize) -> Vec<Brick> {
    let columns = cols as f64;
    let usable_w = (width - MARGIN_X * 2.0 - SPACING * (columns - 1.0)).max(MIN_USABLE_WIDTH);
    let brick_w = usable_w / columns;
    let mut bricks = Vec::with_capacity(rows.saturating_mul(cols));
    for row in 0..rows {
        for col in 0..cols {
            let hue = 200.0 + (row as f64 * 12.0) + (col as f64 * 4.0);
            bricks.push(Brick {
                x: MARGIN_X + col as f64 * (brick_w + SPACING),
                y: TOP + row as f64 * (BRICK_HEIGHT + SPACING),
                w: brick_w,
                h: BRICK_HEIGHT,
                alive: true,
                hue,
                palette: BrickPalette::for_hue(hue),
            });
        }
    }
    bricks
}

#[cfg(test)]
mod tests {
    use super::{BrickPalette, brick_grid};

    #[test]
    fn palette_matches_the_per_frame_format_it_replaces() {
        let palette = BrickPalette::for_hue(212.0);
        assert_eq!(palette.top, "hsla(212, 88%, 68%, 0.95)");
        assert_eq!(palette.bottom, "hsla(212, 80%, 46%, 0.95)");
        assert_eq!(palette.glow, "hsla(212, 90%, 60%, 0.7)");
    }

    #[test]
    fn grid_precomputes_each_brick_palette_from_its_hue() {
        let bricks = brick_grid(1280.0, 6, 10);
        assert_eq!(bricks.len(), 60);
        assert!(bricks.iter().all(|brick| brick.alive));
        assert!(
            bricks
                .iter()
                .all(|brick| brick.palette == BrickPalette::for_hue(brick.hue))
        );
        let last = bricks.last().map(|brick| brick.hue);
        assert_eq!(last, Some(200.0 + 5.0 * 12.0 + 9.0 * 4.0));
    }

    #[test]
    fn grid_keeps_a_minimum_width_on_narrow_viewports() {
        let bricks = brick_grid(100.0, 1, 4);
        let widths = bricks.iter().map(|brick| brick.w).collect::<Vec<_>>();
        assert_eq!(widths, [50.0; 4]);
    }
}
