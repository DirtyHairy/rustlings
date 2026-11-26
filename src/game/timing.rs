use rustlings::sdl::{current_refresh_rate, get_canvas_vsync};
use sdl3::{render::Canvas, video::Window};

const FPS_FALLBACK: f32 = 60.;
const TIME_BUDGET_SAFETY_MARGIN_MUSEC: u64 = 3000;

pub struct RenderConfiguration {
    pub vsync: bool,
    pub real_fps: Option<f32>,
    pub fps: f32,
    pub frame_duration_musec: u64,
    pub time_budget_per_frame_musec: u64,
    pub use_frame_sync: bool,
}

impl RenderConfiguration {
    pub fn query(canvas: &Canvas<Window>) -> Self {
        let vsync = get_canvas_vsync(canvas);
        let real_fps = current_refresh_rate(canvas.window());
        let is_software = canvas.renderer_name.to_lowercase() == "software";

        let fps = if vsync {
            real_fps.unwrap_or(FPS_FALLBACK)
        } else {
            FPS_FALLBACK
        };

        let use_frame_sync = !is_software && real_fps.is_some() && vsync;
        let frame_duration_musec = (1000000. / fps).round() as u64;

        let time_budget_per_frame_musec = if vsync {
            frame_duration_musec.saturating_sub(TIME_BUDGET_SAFETY_MARGIN_MUSEC)
        } else {
            frame_duration_musec
        };

        Self {
            vsync,
            real_fps,
            fps,
            frame_duration_musec,
            time_budget_per_frame_musec,
            use_frame_sync,
        }
    }

    pub fn is_stale(&self, canvas: &Canvas<Window>) -> bool {
        let vsync = get_canvas_vsync(canvas);
        let real_fps = current_refresh_rate(canvas.window());

        self.vsync != vsync || self.real_fps != real_fps
    }

    pub fn clamped_tick_duration_musec(&self, ticks_per_second: u32) -> u64 {
        ((self.fps / ticks_per_second as f32).round() as u64).max(1) * self.frame_duration_musec
    }
}
