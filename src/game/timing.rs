pub fn clamped_tick_duration_musec(ticks_per_second: u32, fps: f32) -> u64 {
    ((fps / ticks_per_second as f32).round() * 1000000. / fps).round() as u64
}
