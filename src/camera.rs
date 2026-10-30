use ggez::{
    context::Has,
    graphics::{Canvas, GraphicsContext, Rect},
};

pub struct Camera {
    /// Default zoom is '1.0'
    pub zoom: f32,
    /// Target is in game coordiantes
    pub target: (f32, f32),
}

impl Camera {
    pub fn new(target: (f32, f32), zoom: f32) -> Self {
        if zoom <= 0.0 {
            panic!("Zoom is smaller than '0.0'");
        }

        Self { zoom, target }
    }

    /// Target will be drawn to the center of the screen
    pub fn to_canvas(&self, gfx: &impl Has<GraphicsContext>) -> Canvas {
        let gfx = gfx.retrieve();

        let mut canvas = Canvas::from_frame(gfx, None);
        let (x, y) = self.target;
        let (w, h) = gfx.drawable_size();
        canvas.set_screen_coordinates(Rect::new(
            -(w / self.zoom / 2.0 - x),
            h / self.zoom / 2.0 - y,
            w / self.zoom,
            -h / self.zoom,
        ));

        return canvas;
    }

    pub fn set_target(&mut self, x: f32, y: f32) {
        self.target = (x, y);
    }
}
