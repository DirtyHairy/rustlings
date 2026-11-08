use std::rc::Rc;

use anyhow::{Error, Result, anyhow};
use bitflags::bitflags;
use rustlings::{
    game_data::{
        GameData, LEVEL_HEIGHT, LEVEL_WIDTH, Level, MINIMAP_AREA_Y, MINIMAP_FRAME_HEIGHT,
        MINIMAP_FRAME_WIDTH, MINIMAP_VIEW_HEIGHT, MINIMAP_VIEW_WIDTH, MINIMAP_VIEW_X,
        MINIMAP_VIEW_Y, SCREEN_HEIGHT, SCREEN_WIDTH, SKILL_PANEL_HEIGHT,
    },
    sdl::{texture_from_bitmap, with_texture_canvas},
};
use sdl3::{
    pixels::{Color, PixelFormat},
    rect::Rect as SdlRect,
    render::{
        BlendMode, Canvas, FPoint, RenderTarget, ScaleMode, Texture, TextureAccess, TextureCreator,
    },
    video::Window,
};

use crate::{
    geometry::Rect,
    scenes::{
        common::{LevelRenderer, TerrainDiff},
        scene_level::{
            cache::Cache, selection_controller::SelectionMode,
            skill_panel_renderer::SkillPanelRenderer,
        },
    },
    state::SceneStateLevel,
};

bitflags! {
    #[derive(Clone, Copy)]
    pub struct Redraw: u32 {
        const LEVEL = 0x01;
        const SCREEN = 0x02;
        const SKILL_PANEL = 0x04;
    }
}

const SKILL_PANEL_Y: u32 = SCREEN_HEIGHT - SKILL_PANEL_HEIGHT;

const TEXTURE_ID_MAIN_SCREEN: usize = 0;
const TEXTURE_ID_MINIMAP: usize = 1;

const MINIMAP_LEMMING_COLOR: Color = Color::RGBA(255, 255, 255, 200);

pub struct Renderer<'texture_creator> {
    redraw: Redraw,

    texture_screen: Texture<'texture_creator>,
    texture_minimap_frame: Texture<'texture_creator>,

    skill_panel_renderer: SkillPanelRenderer<'texture_creator>,
    level_renderer: LevelRenderer<'texture_creator>,

    minimap_points: Vec<FPoint>,
    minimap_points_lookup: Vec<u32>,
}

impl<'texture_creator> Renderer<'texture_creator> {
    pub fn new<T>(
        level: &Level,
        scene_state: &SceneStateLevel,
        game_data: Rc<GameData>,
        texture_creator: &'texture_creator TextureCreator<T>,
    ) -> Result<Self> {
        let palette_skill_panel = game_data.resolve_skill_panel_palette(0);

        let texture_minimap_frame = texture_from_bitmap(
            &game_data.skill_panel.minimap_frame,
            &palette_skill_panel,
            texture_creator,
            TextureAccess::Static,
        )?;

        let texture_screen = texture_creator.create_texture_target(
            PixelFormat::RGBA8888,
            SCREEN_WIDTH,
            SCREEN_HEIGHT,
        )?;

        let skill_panel_renderer =
            SkillPanelRenderer::new(level, Rc::clone(&game_data), texture_creator)?;

        let minimap_pixel_count = MINIMAP_VIEW_WIDTH as usize * MINIMAP_VIEW_HEIGHT as usize;

        let minimap_points: Vec<FPoint> =
            vec![FPoint::new(0., 0.); level.parameters.released as usize];
        let minimap_points_lookup: Vec<u32> =
            vec![0; (minimap_pixel_count as f32 / 32.).ceil() as usize];

        let level_renderer =
            LevelRenderer::new(level, &scene_state.terrain, game_data, texture_creator)?;

        Ok(Renderer {
            redraw: Redraw::all(),

            texture_screen,
            texture_minimap_frame,

            skill_panel_renderer,
            level_renderer,

            minimap_points,
            minimap_points_lookup,
        })
    }

    pub fn apply_diff(&mut self, canvas: &mut Canvas<Window>, diff: &[TerrainDiff]) -> Result<()> {
        self.level_renderer.apply_diff(canvas, diff)?;
        self.mark_for_redraw(Redraw::LEVEL);

        Ok(())
    }

    pub fn mark_for_redraw(&mut self, redraw: Redraw) {
        self.redraw.insert(redraw);
    }

    pub fn will_redraw(&self) -> bool {
        !self.redraw.is_empty()
    }

    pub fn register_layers(&self, compositor: &mut dyn crate::scene::Compositor) {
        compositor.add_layer(
            TEXTURE_ID_MINIMAP,
            LEVEL_WIDTH,
            LEVEL_HEIGHT,
            Rect::new(
                MINIMAP_VIEW_X,
                SKILL_PANEL_Y + MINIMAP_VIEW_Y,
                MINIMAP_VIEW_WIDTH,
                MINIMAP_VIEW_HEIGHT,
            ),
        );

        compositor.add_layer(
            TEXTURE_ID_MAIN_SCREEN,
            SCREEN_WIDTH,
            SCREEN_HEIGHT,
            Rect::new(0, 0, SCREEN_WIDTH, SCREEN_HEIGHT),
        );
    }

    fn minimap_frame_position(&self, state: &SceneStateLevel) -> (u32, u32) {
        (
            MINIMAP_VIEW_X + (state.level_x * MINIMAP_VIEW_WIDTH) / LEVEL_WIDTH - 1,
            SKILL_PANEL_Y + MINIMAP_AREA_Y,
        )
    }

    pub fn texture(&mut self, id: usize) -> Result<&mut Texture<'texture_creator>> {
        match id {
            TEXTURE_ID_MAIN_SCREEN => Ok(&mut self.texture_screen),
            TEXTURE_ID_MINIMAP => Ok(self.level_renderer.texture()),
            _ => Err(anyhow!("invalid texture id {}", id)),
        }
    }

    pub fn draw(
        &mut self,
        state: &SceneStateLevel,
        cache: &mut Cache,
        selection_mode: SelectionMode,
        canvas: &mut Canvas<Window>,
    ) -> Result<bool> {
        if self.redraw.is_empty() {
            return Ok(false);
        }

        let mut redraw = self.redraw;
        self.redraw = Redraw::empty();

        if redraw.contains(Redraw::LEVEL) {
            self.draw_level(state, canvas)?;

            redraw.insert(Redraw::SCREEN);
        }

        if redraw.contains(Redraw::SKILL_PANEL)
            && self
                .skill_panel_renderer
                .draw(state, cache, selection_mode, canvas)?
        {
            redraw.insert(Redraw::SCREEN);
        }

        if redraw.contains(Redraw::SCREEN) {
            self.draw_screen(state, canvas)?;
        }

        Ok(true)
    }

    fn draw_level(&mut self, state: &SceneStateLevel, canvas: &mut Canvas<Window>) -> Result<()> {
        self.level_renderer
            .draw(&state.object_state, &state.lemmings, canvas)
    }

    fn draw_screen(&mut self, state: &SceneStateLevel, canvas: &mut Canvas<Window>) -> Result<()> {
        let (frame_x, frame_y) = self.minimap_frame_position(state);

        with_texture_canvas(canvas, &mut self.texture_screen, |canvas| -> Result<()> {
            canvas.set_draw_color(Color::RGBA(0, 0, 0, 0xff));
            canvas.clear();

            let skill_panel_texture = self.skill_panel_renderer.texture();
            skill_panel_texture.set_blend_mode(BlendMode::None);
            skill_panel_texture.set_scale_mode(ScaleMode::Nearest);
            canvas.copy(
                skill_panel_texture,
                None,
                SdlRect::new(0, LEVEL_HEIGHT as i32, SCREEN_WIDTH, SKILL_PANEL_HEIGHT),
            )?;

            let texture_level = self.level_renderer.texture();

            texture_level.set_blend_mode(BlendMode::None);
            texture_level.set_scale_mode(ScaleMode::Nearest);
            canvas.copy(
                texture_level,
                SdlRect::new(state.level_x as i32, 0, SCREEN_WIDTH, LEVEL_HEIGHT),
                SdlRect::new(0, 0, SCREEN_WIDTH, LEVEL_HEIGHT),
            )?;

            self.minimap_points_lookup.fill(0);
            draw_minimap_lemmings(
                canvas,
                state,
                &mut self.minimap_points,
                &mut self.minimap_points_lookup,
            )?;

            self.texture_minimap_frame.set_blend_mode(BlendMode::Blend);
            self.texture_minimap_frame
                .set_scale_mode(ScaleMode::Nearest);
            canvas.copy(
                &self.texture_minimap_frame,
                None,
                SdlRect::new(
                    frame_x as i32,
                    frame_y as i32,
                    MINIMAP_FRAME_WIDTH,
                    MINIMAP_FRAME_HEIGHT,
                ),
            )?;

            Ok(())
        })?;

        Ok(())
    }
}

fn draw_minimap_lemmings<T: RenderTarget>(
    canvas: &mut Canvas<T>,
    state: &SceneStateLevel,
    minimap_points: &mut [FPoint],
    minimap_points_lookup: &mut [u32],
) -> Result<()> {
    let mut num_points: usize = 0;

    for lemming in &state.lemmings {
        let minimap_x: u32 =
            (lemming.x.clamp(0, LEVEL_WIDTH as i32 - 1) as u32 * MINIMAP_VIEW_WIDTH) / LEVEL_WIDTH;
        let minimap_y: u32 = ((lemming.y - 5).clamp(0, LEVEL_HEIGHT as i32 - 1) as u32
            * MINIMAP_VIEW_HEIGHT)
            / LEVEL_HEIGHT;

        let pixel_index = minimap_x + minimap_y * MINIMAP_VIEW_WIDTH;
        let lookup_index = (pixel_index >> 5) as usize;
        let lookup_mask = 0x01_u32 << (pixel_index & 0x1f);

        if minimap_points_lookup[lookup_index] & lookup_mask == 0 {
            minimap_points[num_points] = FPoint::new(
                (MINIMAP_VIEW_X + minimap_x) as f32,
                (SKILL_PANEL_Y + MINIMAP_VIEW_Y + minimap_y) as f32,
            );
            num_points += 1;

            minimap_points_lookup[lookup_index] |= lookup_mask;
        }
    }

    if num_points == 0 {
        return Ok(());
    }

    canvas.set_draw_color(MINIMAP_LEMMING_COLOR);
    canvas.set_blend_mode(BlendMode::Blend);
    canvas
        .draw_points(&minimap_points[0..num_points])
        .map_err(Error::from)
}
