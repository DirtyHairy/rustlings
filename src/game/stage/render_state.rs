use anyhow::Result;

use rustlings::{
    game_data::{
        Bitmap, CURSOR_CENTER_X, CURSOR_CENTER_Y, CURSOR_NATIVE_ASPECT,
        CURSOR_NATIVE_SCREEN_HEIGHT, CURSOR_NATIVE_SCREEN_WIDTH, CURSOR_SIZE, GameData,
        PALETTE_SIZE, PaletteEntry,
    },
    sdl_rendering::texture_from_bitmap,
};
use sdl3::render::{ScaleMode, Texture, TextureCreator};

use crate::{
    geometry,
    scene::{Compositor, CursorType, Scene},
};

#[derive(Debug, PartialEq, Clone, Copy)]
pub enum PrescalingMode {
    None(ScaleMode),
    Quis(usize, usize),
}

impl Default for PrescalingMode {
    fn default() -> Self {
        PrescalingMode::None(ScaleMode::Nearest)
    }
}

pub struct Layer<'texture_creator> {
    pub texture_id: usize,

    pub texture_width: usize,
    pub texture_height: usize,
    pub destination: geometry::Rect,

    pub prescaling_mode: PrescalingMode,
    pub current_prescaling_mode: PrescalingMode,
    pub intermediate_texture: Option<Texture<'texture_creator>>,
}

#[derive(Default, Clone)]
pub struct CursorLayout {
    pub width: usize,
    pub height: usize,

    pub center_x: usize,
    pub center_y: usize,

    pub prescaling_mode: PrescalingMode,
}

#[derive(Default)]
pub struct Layout {
    width: usize,
    height: usize,

    pub scene: geometry::Rect,
    pub layers: Vec<geometry::Rect>,
    pub cursor: CursorLayout,
}

pub struct StaticTexture<'texture_creator> {
    pub texture: Texture<'texture_creator>,
    pub intermediate_texture: Option<Texture<'texture_creator>>,

    pub prescaling_mode: PrescalingMode,
}

impl<'texture_creator> StaticTexture<'texture_creator> {
    pub fn new<T>(
        bitmap: &Bitmap,
        palette: &[PaletteEntry; PALETTE_SIZE],
        texture_creator: &'texture_creator TextureCreator<T>,
    ) -> Result<Self> {
        Ok(StaticTexture {
            texture: texture_from_bitmap(bitmap, palette, texture_creator)?,
            intermediate_texture: None,
            prescaling_mode: PrescalingMode::None(ScaleMode::Nearest),
        })
    }
}

pub struct Cursors<'texture_creator> {
    pub crosshair: StaticTexture<'texture_creator>,
    pub boxx: StaticTexture<'texture_creator>,
}

impl<'texture_creator> Cursors<'texture_creator> {
    pub fn new<T>(
        game_data: &GameData,
        texture_creator: &'texture_creator TextureCreator<T>,
    ) -> Result<Self> {
        Ok(Cursors {
            crosshair: StaticTexture::new(
                &game_data.cursors.crosshair,
                &game_data.static_palette,
                texture_creator,
            )?,
            boxx: StaticTexture::new(
                &game_data.cursors.boxx,
                &game_data.static_palette,
                texture_creator,
            )?,
        })
    }
}

pub struct RenderState<'texture_creator> {
    pub layers: Vec<Layer<'texture_creator>>,
    pub cursors: Cursors<'texture_creator>,
    pub layout: Layout,

    pub scene_width: usize,
    pub scene_height: usize,
    pub scene_aspect: f32,

    pub mouse_x: f32,
    pub mouse_y: f32,
    pub mouse_enabled: bool,
}

impl<'texture_creator> RenderState<'texture_creator> {
    pub fn new<T>(
        scene: &dyn Scene,
        game_data: &GameData,
        texture_creator: &'texture_creator TextureCreator<T>,
    ) -> Result<Self> {
        Ok(RenderState {
            layers: Vec::new(),
            cursors: Cursors::new(game_data, texture_creator)?,
            layout: Default::default(),
            scene_width: scene.width(),
            scene_height: scene.height(),
            scene_aspect: scene.aspect(),
            mouse_x: 0.,
            mouse_y: 0.,
            mouse_enabled: false,
        })
    }

    pub fn get_cursor(
        &mut self,
        cursor: CursorType,
    ) -> Option<&mut StaticTexture<'texture_creator>> {
        match cursor {
            CursorType::None => None,
            CursorType::Crosshair => Some(&mut self.cursors.crosshair),
            CursorType::Box => Some(&mut self.cursors.boxx),
        }
    }

    pub fn update_layout(&mut self, width: usize, height: usize) {
        if self.layout.width == width && self.layout.height == height {
            return;
        }

        self.layout.width = width;
        self.layout.height = height;

        self.layout_scene(width, height);

        let scale_x = self.layout.scene.width as f32 / self.scene_width as f32;
        let scale_y = self.layout.scene.height as f32 / self.scene_height as f32;

        self.layout_layers(scale_x, scale_y);
        self.layout_cursor(scale_x, scale_y);
    }

    fn layout_scene(&mut self, width: usize, height: usize) {
        let w = width as f32;
        let h = height as f32;
        let w_scene = self.scene_width as f32;
        let h_scene = self.scene_height as f32 * self.scene_aspect;

        if w_scene * h / h_scene <= w {
            let width = w_scene * h / h_scene;

            self.layout.scene.height = height;
            self.layout.scene.width = width.round() as usize;
            self.layout.scene.y = 0;
            self.layout.scene.x = ((w - width) / 2.).round() as usize;
        } else {
            let height = h_scene * w / w_scene;

            self.layout.scene.width = width;
            self.layout.scene.height = height.round() as usize;
            self.layout.scene.x = 0;
            self.layout.scene.y = ((h - height) / 2.).round() as usize;
        }
    }

    fn layout_layers(&mut self, scale_x: f32, scale_y: f32) {
        self.layout
            .layers
            .reserve_exact(self.layers.len() - self.layout.layers.len());
        self.layout.layers.clear();

        for layer in &mut self.layers {
            let dest = geometry::Rect {
                x: self.layout.scene.x + (layer.destination.x as f32 * scale_x).round() as usize,
                y: self.layout.scene.y + (layer.destination.y as f32 * scale_y).round() as usize,
                width: (layer.destination.width as f32 * scale_x).round() as usize,
                height: (layer.destination.height as f32 * scale_y).round() as usize,
            };

            self.layout.layers.push(dest);
            layer.prescaling_mode =
                calculate_prescaling_mode(layer.texture_width, layer.texture_height, &dest);
        }
    }

    fn layout_cursor(&mut self, scale_x: f32, scale_y: f32) {
        let cursor_scale_x = scale_x * self.scene_width as f32 / CURSOR_NATIVE_SCREEN_WIDTH as f32;
        let cursor_scale_y = scale_y * self.scene_height as f32
            / CURSOR_NATIVE_SCREEN_HEIGHT as f32
            * self.scene_aspect
            / CURSOR_NATIVE_ASPECT;

        self.layout.cursor.width = (cursor_scale_x * CURSOR_SIZE as f32).round() as usize;
        self.layout.cursor.height = (cursor_scale_y * CURSOR_SIZE as f32).round() as usize;
        self.layout.cursor.center_x = (cursor_scale_x * CURSOR_CENTER_X).round() as usize;
        self.layout.cursor.center_y = (cursor_scale_y * CURSOR_CENTER_Y).round() as usize;
        self.layout.cursor.prescaling_mode = calculate_prescaling_mode(
            CURSOR_SIZE,
            CURSOR_SIZE,
            &geometry::Rect::new(0, 0, self.layout.cursor.width, self.layout.cursor.height),
        );
    }
}

fn calculate_prescaling_mode(width: usize, height: usize, dest: &geometry::Rect) -> PrescalingMode {
    if width == 0 || height == 0 {
        return Default::default();
    }
    let mut integer_scale_x = (dest.width as f32 / width as f32).round() as usize;
    let mut integer_scale_y = (dest.height as f32 / height as f32).round() as usize;

    if (integer_scale_x == 0 && integer_scale_y <= 1)
        || (integer_scale_y == 0 && integer_scale_x <= 1)
    {
        // At least one axis is downscaled, and the other is not nontrivially
        // integer scaled -> use the original texture and use linear scaling
        return PrescalingMode::None(ScaleMode::Linear);
    }

    // We are integer scaling along at least one axis, so make sure we keep
    // the other finite.
    integer_scale_x = std::cmp::max(1, integer_scale_x);
    integer_scale_y = std::cmp::max(1, integer_scale_y);

    let integer_scaled_width = width * integer_scale_x;
    let integer_scaled_height = height * integer_scale_y;

    if integer_scaled_width == dest.width && integer_scaled_height == dest.height {
        // Integer scaling step is sufficient -> use the original texture and use
        // nearest-neighbour scaling.
        return PrescalingMode::None(ScaleMode::Nearest);
    }

    if integer_scale_x == 1 && integer_scale_y == 1 {
        // The integer scaling step is trivial -> use the original texture
        // and use linear scaling
        return PrescalingMode::None(ScaleMode::Linear);
    }

    PrescalingMode::Quis(integer_scaled_width, integer_scaled_height)
}

impl Compositor for RenderState<'_> {
    fn add_layer(
        &mut self,
        texture_id: usize,
        width: usize,
        height: usize,
        destination: geometry::Rect,
    ) {
        self.layers.push(Layer {
            texture_id,
            texture_width: width,
            texture_height: height,
            destination,
            intermediate_texture: None,
            prescaling_mode: Default::default(),
            current_prescaling_mode: Default::default(),
        });
    }
}

#[cfg(test)]
mod test {
    use sdl3::render::ScaleMode;

    use crate::{
        geometry,
        stage::render_state::{PrescalingMode, calculate_prescaling_mode},
    };

    #[test]
    fn calculate_prescaling_mode_degenerate_width() {
        let prescaling_mode =
            calculate_prescaling_mode(0, 100, &geometry::Rect::new(0, 0, 100, 100));

        assert_eq!(prescaling_mode, Default::default());
    }

    #[test]
    fn calculate_prescaling_mode_degenerate_height() {
        let prescaling_mode =
            calculate_prescaling_mode(100, 0, &geometry::Rect::new(0, 0, 100, 100));

        assert_eq!(prescaling_mode, Default::default());
    }

    #[test]
    fn calculate_prescaling_mode_downscale_both_1() {
        let prescaling_mode =
            calculate_prescaling_mode(100, 100, &geometry::Rect::new(0, 0, 20, 20));

        assert_eq!(prescaling_mode, PrescalingMode::None(ScaleMode::Linear));
    }

    #[test]
    fn calculate_prescaling_mode_downscale_both_2() {
        let prescaling_mode =
            calculate_prescaling_mode(100, 100, &geometry::Rect::new(0, 0, 90, 90));

        assert_eq!(prescaling_mode, PrescalingMode::None(ScaleMode::Linear));
    }

    #[test]
    fn calculate_prescaling_mode_downscale_one_1() {
        let prescaling_mode =
            calculate_prescaling_mode(100, 100, &geometry::Rect::new(0, 0, 90, 190));

        assert_eq!(prescaling_mode, PrescalingMode::Quis(100, 200));
    }

    #[test]
    fn calculate_prescaling_mode_downscale_one_2() {
        let prescaling_mode =
            calculate_prescaling_mode(100, 100, &geometry::Rect::new(0, 0, 90, 110));

        assert_eq!(prescaling_mode, PrescalingMode::None(ScaleMode::Linear));
    }

    #[test]
    fn calculate_prescaling_mode_upscale_exact() {
        let prescaling_mode =
            calculate_prescaling_mode(100, 100, &geometry::Rect::new(0, 0, 200, 300));

        assert_eq!(prescaling_mode, PrescalingMode::None(ScaleMode::Nearest));
    }

    #[test]
    fn calculate_prescaling_mode_upscale_slighty() {
        let prescaling_mode =
            calculate_prescaling_mode(100, 100, &geometry::Rect::new(0, 0, 110, 120));

        assert_eq!(prescaling_mode, PrescalingMode::None(ScaleMode::Linear))
    }

    #[test]
    fn calculate_prescaling_mode_upscale_almost() {
        let prescaling_mode =
            calculate_prescaling_mode(100, 100, &geometry::Rect::new(0, 0, 110, 190));

        assert_eq!(prescaling_mode, PrescalingMode::Quis(100, 200));
    }

    #[test]
    fn calculate_prescaling_mode_upscale() {
        let prescaling_mode =
            calculate_prescaling_mode(100, 100, &geometry::Rect::new(0, 0, 610, 890));

        assert_eq!(prescaling_mode, PrescalingMode::Quis(600, 900));
    }
}
