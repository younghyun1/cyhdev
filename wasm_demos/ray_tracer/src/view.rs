//! Viewport sizing and camera invalidation shared by the GPU and CPU renderers.
//! Kept free of browser types so the rules run under host unit tests.

/// Upper bound on backing-store pixels per CSS pixel. Tracing cost grows with
/// the square of this scale, and past 1.5 the extra detail is hard to see on
/// a progressively refined image, so a 3x display is capped rather than
/// paying four times the per-frame work.
pub(crate) const RENDER_SCALE_MAX: f64 = 1.5;

/// Backing-store pixels per CSS pixel for a device pixel ratio.
pub(crate) fn render_scale(device_pixel_ratio: f64) -> f64 {
    device_pixel_ratio.min(RENDER_SCALE_MAX)
}

/// Backing-store size for a viewport measured in CSS pixels, at least 1x1.
pub(crate) fn render_size(css_width: f64, css_height: f64, device_pixel_ratio: f64) -> (u32, u32) {
    let scale = render_scale(device_pixel_ratio);
    // `as` saturates and maps NaN to 0, which the lower bound then lifts to 1.
    let width = (css_width * scale).max(1.0) as u32;
    let height = (css_height * scale).max(1.0) as u32;
    (width, height)
}

/// Returns the new backing-store size only when it differs from `current`.
/// Assigning a canvas width or height clears the canvas and resets its 2D
/// context state even when the value is unchanged, so callers write the DOM
/// only for `Some`.
pub(crate) fn resized(
    current: (u32, u32),
    css_width: f64,
    css_height: f64,
    device_pixel_ratio: f64,
) -> Option<(u32, u32)> {
    let next = render_size(css_width, css_height, device_pixel_ratio);
    (next != current).then_some(next)
}

/// Coalesces camera changes so accumulated samples are discarded at most once
/// per animation frame, however many pointer events arrive in between. Each
/// reset clears and reseeds buffers proportional to the pixel count.
#[derive(Debug, Default)]
pub(crate) struct CameraInvalidation {
    pending: bool,
}

impl CameraInvalidation {
    /// Records that the camera moved since the last frame.
    pub(crate) fn mark(&mut self) {
        self.pending = true;
    }

    /// Returns whether a reset is due and clears the pending flag.
    pub(crate) fn take(&mut self) -> bool {
        std::mem::take(&mut self.pending)
    }
}

#[cfg(test)]
mod tests {
    use super::{CameraInvalidation, RENDER_SCALE_MAX, render_scale, render_size, resized};

    #[test]
    fn render_scale_caps_high_density_displays() {
        assert_eq!(render_scale(1.0), 1.0);
        assert_eq!(render_scale(1.25), 1.25);
        assert_eq!(render_scale(3.0), RENDER_SCALE_MAX);
        assert_eq!(render_size(1280.0, 720.0, 3.0), (1920, 1080));
        assert_eq!(render_size(1280.0, 720.0, 1.0), (1280, 720));
    }

    #[test]
    fn render_size_never_reaches_zero() {
        assert_eq!(render_size(0.0, 0.0, 2.0), (1, 1));
        assert_eq!(render_size(f64::NAN, 10.0, 1.0), (1, 10));
    }

    #[test]
    fn resize_reports_only_real_size_changes() {
        assert_eq!(resized((1920, 1080), 1280.0, 720.0, 2.0), None);
        assert_eq!(resized((1920, 1080), 1280.0, 720.0, 1.0), Some((1280, 720)));
        assert_eq!(resized((1280, 720), 1440.0, 900.0, 1.0), Some((1440, 900)));
    }

    #[test]
    fn camera_invalidation_resets_once_per_frame() {
        let mut invalidation = CameraInvalidation::default();
        assert!(!invalidation.take());
        for _ in 0..12 {
            invalidation.mark();
        }
        assert!(invalidation.take());
        assert!(!invalidation.take());
    }
}
