use crate::{
    trezorhal::display,
    ui::{
        display::Color,
        geometry::Offset,
        shape::{
            render::ScopedRenderer, BasicCanvas, DirectRenderer, DrawingCache, Rgb565Canvas,
            Viewport,
        },
    },
};

#[cfg(feature = "ui_debug_overlay")]
use crate::{
    trezorhal::time,
    ui::{CommonUI, DebugOverlay, ModelUI},
};

use super::bumps;

pub type ConcreteRenderer<'a, 'alloc> = DirectRenderer<'a, 'alloc, Rgb565Canvas<'alloc>>;

// Time of the last frame buffer get operation
#[cfg(feature = "ui_debug_overlay")]
static mut FRAME_BUFFER_GET_TIME: u64 = 0;

/// Creates the `Renderer` object for drawing on a display and invokes a
/// user-defined function that takes a single argument `target`. The user's
/// function can utilize the `target` for drawing on the display.
///
/// `clip` specifies a rectangle area that the user will draw to.
/// If no clip is specified, the entire display area is used.
///
/// `bg_color` specifies a background color with which the clip is filled before
/// the drawing starts. If the background color is None, the background
/// is undefined, and the user has to fill it themselves.
pub fn render_on_display<'env, F>(viewport: Option<Viewport>, bg_color: Option<Color>, func: F)
where
    F: for<'alloc> FnOnce(&mut ScopedRenderer<'alloc, 'env, ConcreteRenderer<'_, 'alloc>>),
{
    bumps::run_with_bumps(|bump_a, bump_b| {
        let width = display::DISPLAY_RESX as i16;
        let height = display::DISPLAY_RESY as i16;

        let cache = DrawingCache::new(bump_a, bump_b);

        #[cfg(feature = "ui_debug_overlay")]
        let refresh_time = unsafe { time::ticks_us() - FRAME_BUFFER_GET_TIME };

        let fb_info = display::get_frame_buffer();

        #[cfg(feature = "ui_debug_overlay")]
        unsafe {
            FRAME_BUFFER_GET_TIME = time::ticks_us()
        };

        if fb_info.is_none() {
            return;
        }

        let (fb, fb_stride) = fb_info.unwrap();

        let mut canvas = unwrap!(Rgb565Canvas::new(
            Offset::new(width, height),
            Some(fb_stride),
            None,
            fb
        ));

        if let Some(viewport) = viewport {
            canvas.set_viewport(viewport);
        }

        let mut target = ScopedRenderer::new(DirectRenderer::new(&mut canvas, bg_color, &cache));

        // In debug mode, measure the time spent on rendering.
        #[cfg(feature = "ui_debug_overlay")]
        {
            let render_time = time::measure_us(|| func(&mut target));
            let info = DebugOverlay {
                render_time,
                refresh_time,
            };
            ModelUI::render_debug_overlay(&mut target, info);
        }

        // In production, just execute the drawing function without timing.
        #[cfg(not(feature = "ui_debug_overlay"))]
        {
            func(&mut target);
        }
    });
}
