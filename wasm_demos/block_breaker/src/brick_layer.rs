//! Cached brick rendering. Each brick is drawn with a gradient and a 22px
//! shadow blur, the costliest canvas work in a frame, yet bricks change only
//! when a level is laid out, a brick breaks, or the canvas resizes. The layer
//! is redrawn on a detached canvas after such a change and copied onto the
//! visible canvas every frame.

use wasm_bindgen::{JsCast, JsValue};
use web_sys::{CanvasRenderingContext2d, Document, HtmlCanvasElement};

use crate::bricks::Brick;

/// Canvas shadow blur ignores the current transform, so this is in backing
/// pixels, exactly as when bricks were drawn on the visible canvas.
const BRICK_GLOW_BLUR: f64 = 22.0;

/// Whether the cached layer no longer matches the bricks. It starts stale
/// because nothing has been drawn yet.
#[derive(Debug)]
pub(crate) struct Staleness {
    stale: bool,
}

impl Default for Staleness {
    fn default() -> Self {
        Self { stale: true }
    }
}

impl Staleness {
    pub(crate) fn mark(&mut self) {
        self.stale = true;
    }

    /// Returns whether a redraw is due and records that it is being done.
    pub(crate) fn take(&mut self) -> bool {
        std::mem::take(&mut self.stale)
    }
}

pub(crate) struct BrickLayer {
    canvas: HtmlCanvasElement,
    ctx: CanvasRenderingContext2d,
    staleness: Staleness,
}

impl BrickLayer {
    pub(crate) fn new(document: &Document) -> Result<Self, JsValue> {
        let canvas = document
            .create_element("canvas")?
            .dyn_into::<HtmlCanvasElement>()?;
        let ctx = canvas
            .get_context("2d")?
            .ok_or_else(|| JsValue::from_str("no brick layer context"))?
            .dyn_into::<CanvasRenderingContext2d>()?;
        Ok(Self {
            canvas,
            ctx,
            staleness: Staleness::default(),
        })
    }

    /// Matches the visible canvas backing store and CSS-to-backing transform.
    /// Resizing a canvas clears it, so the layer is redrawn afterward.
    pub(crate) fn resize(&mut self, backing_width: u32, backing_height: u32, scale: f64) {
        self.canvas.set_width(backing_width);
        self.canvas.set_height(backing_height);
        let _ = self.ctx.set_transform(scale, 0.0, 0.0, scale, 0.0, 0.0);
        self.staleness.mark();
    }

    /// Records that a brick appeared, broke, or moved.
    pub(crate) fn invalidate(&mut self) {
        self.staleness.mark();
    }

    /// Redraws every live brick if anything changed since the last redraw.
    /// `width` and `height` are the viewport in CSS pixels.
    pub(crate) fn redraw_if_stale(&mut self, bricks: &[Brick], width: f64, height: f64) {
        if !self.staleness.take() {
            return;
        }
        let ctx = &self.ctx;
        ctx.clear_rect(0.0, 0.0, width, height);
        ctx.save();
        ctx.set_shadow_blur(BRICK_GLOW_BLUR);
        for brick in bricks.iter().filter(|brick| brick.alive) {
            let gradient = ctx.create_linear_gradient(brick.x, brick.y, brick.x, brick.y + brick.h);
            gradient.add_color_stop(0.0, &brick.palette.top).ok();
            gradient.add_color_stop(1.0, &brick.palette.bottom).ok();
            ctx.set_fill_style_canvas_gradient(&gradient);
            ctx.set_shadow_color(&brick.palette.glow);
            ctx.fill_rect(brick.x, brick.y, brick.w, brick.h);

            ctx.set_shadow_blur(0.0);
            ctx.set_fill_style_str("rgba(255,255,255,0.25)");
            ctx.fill_rect(brick.x + 2.0, brick.y + 2.0, brick.w - 4.0, 4.0);
            ctx.set_shadow_blur(BRICK_GLOW_BLUR);
        }
        ctx.restore();
    }

    /// Copies the layer onto `target`, whose transform maps the `width` by
    /// `height` CSS viewport onto the same backing size as this layer. A
    /// failed copy only drops bricks from this frame; the next frame copies
    /// again.
    pub(crate) fn blit(&self, target: &CanvasRenderingContext2d, width: f64, height: f64) {
        let _ = target.draw_image_with_html_canvas_element_and_dw_and_dh(
            &self.canvas,
            0.0,
            0.0,
            width,
            height,
        );
    }
}

#[cfg(test)]
mod tests {
    use super::Staleness;

    #[test]
    fn layer_starts_stale_and_redraws_once_per_change_burst() {
        let mut staleness = Staleness::default();
        assert!(staleness.take());
        assert!(!staleness.take());
        staleness.mark();
        staleness.mark();
        assert!(staleness.take());
        assert!(!staleness.take());
    }
}
