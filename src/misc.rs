use ggez::{
    Context, GameResult,
    glam::Vec2,
    graphics::{self, Canvas, Color, Mesh},
    input::gamepad::gilrs,
};

const ANALOGUE_STICK_ORIGIN: [f32; 2] = [600.0, 380.0];
const ANALOGUE_STICK_EYE_SOCKET_RADIUS: f32 = 64.0;

pub struct ControllerInputDisplayer {
    analogue_stick_eye_socket_mesh: Mesh,
    analogue_stick_location: [f32; 2],
}

impl ControllerInputDisplayer {
    pub fn new(ctx: &mut Context) -> GameResult<Self> {
        let mb = &mut graphics::MeshBuilder::new();

        mb.circle(
            graphics::DrawMode::stroke(20.0),
            Vec2::from(ANALOGUE_STICK_ORIGIN),
            ANALOGUE_STICK_EYE_SOCKET_RADIUS,
            1.0,
            Color::new(0.24, 0.24, 0.24, 1.0),
        )?;

        let analogue_stick_eye_socket_mesh = Mesh::from_data(ctx, mb.build());

        Ok(Self {
            analogue_stick_eye_socket_mesh,
            analogue_stick_location: [0.0, 0.0],
        })
    }

    pub fn update(&mut self, ctx: &mut Context) {
        while let Some(gilrs::Event { event, .. }) = ctx.gamepad.next_event() {
            let gilrs::EventType::AxisChanged(axis, value, _) = event else {
                continue;
            };

            let i = match axis {
                gilrs::Axis::LeftStickX => 0,
                gilrs::Axis::LeftStickY => 1,
                _ => continue,
            };

            self.analogue_stick_location[i] = value;
        }
    }

    pub fn draw(&self, ctx: &mut Context) -> GameResult {
        let mut canvas = Canvas::from_frame(ctx, None);

        canvas.draw(
            &self.analogue_stick_eye_socket_mesh,
            graphics::DrawParam::new(),
        );

        let mut mb = graphics::MeshBuilder::new();
        mb.polyline(
            graphics::DrawMode::Stroke(
                graphics::StrokeOptions::DEFAULT
                    .with_line_cap(graphics::LineCap::Round)
                    .with_line_width(10.0),
            ),
            &[
                Vec2::from(ANALOGUE_STICK_ORIGIN),
                Vec2::from(ANALOGUE_STICK_ORIGIN)
                    + Vec2::from([
                        self.analogue_stick_location[0],
                        self.analogue_stick_location[1] * -1.0,
                    ]) * ANALOGUE_STICK_EYE_SOCKET_RADIUS,
            ],
            Color::new(1.0, 0.0, 0.0, 1.0),
        )?;

        canvas.draw(
            &graphics::Mesh::from_data(ctx, mb.build()),
            graphics::DrawParam::default(),
        );

        Ok(())
    }
}
