mod camera;
mod hitbox;
mod misc;
mod world;

use ggez::{
    Context, GameResult,
    conf::{WindowMode, WindowSetup},
    event,
    glam::*,
    graphics::{
        self, BlendMode, Canvas, Color, DrawMode, DrawParam, FillOptions, Image, Mesh, Quad,
        TextAlign, TextLayout,
    },
    input::keyboard::KeyCode,
};
use parry2d::{
    math::Pose,
    shape::{Compound, ConvexPolygon, SharedShape},
};

use std::{env, path};

use crate::{
    camera::Camera,
    world::{PLAYER_SIZE, PlayerState},
};

struct MainState {
    camera: Camera,

    // Textures
    stamina_bar: Image,

    frame_counter: u64,
    start_time: std::time::Instant,

    player_state: PlayerState,
    stage: Compound,
}

impl MainState {
    fn new(ctx: &mut Context) -> GameResult<MainState> {
        let stamina_bar = Image::from_path(ctx, "/stamina-bar.png")?;

        Ok(MainState {
            camera: Camera::new((0.0, 0.0), 0.5),
            stamina_bar,
            frame_counter: 0,
            start_time: std::time::Instant::now(),
            player_state: PlayerState::new(0.0, 200.0),
            stage: Compound::new(vec![
                (
                    Pose::IDENTITY,
                    SharedShape::new(
                        ConvexPolygon::from_convex_hull(&[
                            parry2d::math::Vec2::new(-500.0, 0.0),
                            parry2d::math::Vec2::new(-250.0, -250.0),
                            parry2d::math::Vec2::new(250.0, -250.0),
                            parry2d::math::Vec2::new(500.0, 0.0),
                            parry2d::math::Vec2::new(0.0, 100.0),
                        ])
                        .unwrap(),
                    ),
                ),
                (
                    Pose::IDENTITY,
                    SharedShape::new(
                        ConvexPolygon::from_convex_hull(&[
                            parry2d::math::Vec2::new(-500.0, 0.0),
                            parry2d::math::Vec2::new(-500.0, 250.0),
                            parry2d::math::Vec2::new(-350.0, 0.0),
                        ])
                        .unwrap(),
                    ),
                ),
                (
                    Pose::IDENTITY,
                    SharedShape::new(
                        ConvexPolygon::from_convex_hull(&[
                            parry2d::math::Vec2::new(500.0, 0.0),
                            parry2d::math::Vec2::new(500.0, 250.0),
                            parry2d::math::Vec2::new(350.0, 0.0),
                        ])
                        .unwrap(),
                    ),
                ),
            ]),
        })
    }

    fn draw_stamina_bar(
        &self,
        ctx: &mut Context,
        x: f32,
        y: f32,
        spare_stamina: u8,
        recovery_frames: u8,
    ) -> GameResult {
        const MAX_STAMINA_RECT_X: f32 = 8.0;
        const MAX_STAMINA_RECT_Y: f32 = 6.0;
        const _MAX_STAMINA_RECT_W: f32 = 120.0;
        const MAX_STAMINA_RECT_H: f32 = 30.0;

        let mut canvas = Canvas::from_frame(ctx, None);
        canvas.draw(&self.stamina_bar, DrawParam::new().dest([x, y]));

        let spare_stamina_draw_param = DrawParam::new()
            .dest([x + MAX_STAMINA_RECT_X, y + MAX_STAMINA_RECT_Y])
            .scale([spare_stamina as f32, MAX_STAMINA_RECT_H])
            .color(Color::YELLOW);

        let recovery_frames_draw_param = DrawParam::new()
            .dest([
                x + MAX_STAMINA_RECT_X + spare_stamina as f32 - recovery_frames as f32,
                y + MAX_STAMINA_RECT_Y,
            ])
            .scale([recovery_frames as f32, MAX_STAMINA_RECT_H])
            .color(Color::RED);

        canvas.draw(&Quad, spare_stamina_draw_param);
        canvas.draw(&Quad, recovery_frames_draw_param);

        canvas.finish(ctx)?;
        Ok(())
    }

    fn draw_player(&self, ctx: &mut Context) -> GameResult {
        let mut canvas = self.camera.to_canvas(ctx);
        let color = if self.player_state.is_grounded() {
            Color::RED
        } else {
            Color::GREEN
        };

        let circle = Mesh::new_circle(
            ctx,
            DrawMode::Fill(FillOptions::DEFAULT),
            [self.player_state.position.x, self.player_state.position.y],
            PLAYER_SIZE,
            0.1,
            color,
        )?;

        canvas.draw(&circle, DrawParam::new());

        canvas.finish(ctx)?;

        Ok(())
    }
}

impl event::EventHandler for MainState {
    fn update(&mut self, ctx: &mut Context) -> GameResult {
        const DESIRED_FPS: u32 = 60;

        if !ctx.time.check_update_time(DESIRED_FPS) {
            return Ok(());
        }

        let left = ctx.keyboard.is_key_pressed(KeyCode::Left);
        let right = ctx.keyboard.is_key_pressed(KeyCode::Right);
        let direction = match (left, right) {
            (true, true) | (false, false) => 0.0,
            (true, false) => -1.0,
            (false, true) => 1.0,
        };

        self.player_state.set_horizontal_movement(direction);
        if ctx.keyboard.is_key_just_pressed(KeyCode::Space) {
            self.player_state.jump();
        }

        self.player_state.step(&self.stage);

        self.frame_counter += 1;

        Ok(())
    }

    fn draw(&mut self, ctx: &mut Context) -> GameResult {
        let mut canvas = Canvas::from_frame(ctx, Color::new(0.5, 0.5, 0.5, 1.0));

        let mut fps_text = graphics::Text::new(format!(
            "{} frames\n{} seconds",
            self.frame_counter,
            self.start_time.elapsed().as_secs_f32()
        ));

        fps_text.set_layout(TextLayout {
            h_align: TextAlign::End,
            v_align: TextAlign::Begin,
        });

        canvas.draw(&fps_text, DrawParam::new().dest([ctx.gfx.size().0, 0.0]));

        canvas.finish(ctx)?;

        for hitbox in self.stage.shapes() {
            draw_hitbox(
                &self.camera,
                ctx,
                &hitbox.1.as_convex_polygon().unwrap().points(),
                Color::BLUE,
            )?;
        }

        self.draw_stamina_bar(ctx, 50.0, 50.0, 10, 60)?;
        self.draw_player(ctx)?;

        Ok(())
    }
}

fn draw_hitbox(
    camera: &Camera,
    ctx: &mut Context,
    hitbox: &[parry2d::math::Vec2],
    color: Color,
) -> GameResult {
    let mut canvas = camera.to_canvas(ctx);

    let mut mb = graphics::MeshBuilder::new();

    let fill_mode = DrawMode::Fill(FillOptions::DEFAULT);
    let stroke_mode = DrawMode::Stroke(
        graphics::StrokeOptions::DEFAULT
            .with_line_join(graphics::LineJoin::Round)
            .with_line_cap(graphics::LineCap::Round)
            .with_line_width(0.0), // (hitbox.padding()),
    );

    let points: Vec<Vec2> = hitbox
        .iter()
        .chain([&hitbox[0]])
        .map(|&v| Vec2::new(v.x, v.y))
        .collect();

    mb.polyline(fill_mode, &points[..points.len() - 1], color)?;
    mb.polyline(stroke_mode, &points, color)?;

    canvas.set_blend_mode(BlendMode::REPLACE);

    canvas.draw(
        &graphics::Mesh::from_data(ctx, mb.build()),
        DrawParam::new(),
    );

    canvas.finish(ctx)?;

    Ok(())
}

fn main() -> GameResult {
    let resource_dir = if let Ok(manifest_dir) = env::var("CARGO_MANIFEST_DIR") {
        let mut path = path::PathBuf::from(manifest_dir);
        path.push("resources");
        path
    } else {
        path::PathBuf::from("./resources")
    };

    let cb = ggez::ContextBuilder::new("drawing", "ggez")
        .add_resource_path(resource_dir)
        .window_setup(WindowSetup::default().title("Nameless Fighting Game"))
        .window_mode(WindowMode::default().resizable(true));

    let (mut ctx, events_loop) = cb.build()?;

    let state = MainState::new(&mut ctx).unwrap();
    event::run(ctx, events_loop, state)
}
