use std::f32::consts::PI;

use parry2d::math::{Pose, Vec2};
use parry2d::query::{ShapeCastOptions, cast_shapes};
use parry2d::shape;

pub const PLAYER_SIZE: f32 = 36.0;

const MAX_FALL_SPEED: f32 = 30.0;
const BASE_GRAVITY: f32 = 5.0;
const RUN_SPEED: f32 = 24.0;
const HORIZONTAL_AIR_SPEED: f32 = 12.0;
const INITIAL_JUMP_SPEED: f32 = 50.0;

/// i.e. If the player is on the stage then what makes them not slip
const MAX_GROUNDING_ANGLE: f32 = 0.6 * PI;
const MIN_GROUNDING_ANGLE: f32 = PI - MAX_GROUNDING_ANGLE;

/// i.e. If the player jumps below the stage then what makes them stop
const MAX_UPWARD_BLOCKING_ANGLE: f32 = -0.4 * PI;
const MIN_UPWARD_BLOCKING_ANGLE: f32 = -PI - MAX_UPWARD_BLOCKING_ANGLE;

const SKIN_WIDTH: f32 = 1.0;

#[derive(Clone)]
pub struct PlayerState {
    pub position: Vec2,
    pub velocity: Vec2,
    is_grounded: bool,
}

impl PlayerState {
    pub fn new(x: f32, y: f32) -> Self {
        Self {
            position: Vec2::new(x, y),
            velocity: Vec2::ZERO,
            is_grounded: false,
        }
    }

    pub fn is_grounded(&self) -> bool {
        self.is_grounded
    }

    pub fn step(&mut self, stage: &shape::Compound) {
        let output = collide_and_slide(
            &shape::Ball::new(PLAYER_SIZE),
            self.position,
            self.velocity,
            stage,
            0,
        );

        self.position += output.final_velocity;

        self.gravity_step();

        self.is_grounded = output.is_grounded;
    }

    fn gravity_step(&mut self) {
        self.velocity.y -= BASE_GRAVITY;
        self.velocity.y = self.velocity.y.max(-MAX_FALL_SPEED);
    }

    pub fn jump(&mut self) {
        // if !self.is_grounded {
        //     return;
        // }

        self.velocity.y = INITIAL_JUMP_SPEED;
        self.is_grounded = false;
    }

    pub fn set_horizontal_movement(&mut self, direction: f32) {
        let velocity = if self.is_grounded {
            RUN_SPEED
        } else {
            HORIZONTAL_AIR_SPEED
        };

        let direction = if direction < 0.0 {
            -1.0
        } else if direction > 0.0 {
            1.0
        } else {
            0.0
        };

        self.velocity.x = velocity * direction;
    }
}

pub struct CollideAndSlideOutput {
    pub final_velocity: Vec2,
    pub is_grounded: bool,
}

pub fn collide_and_slide(
    player_shape: &impl shape::Shape,
    pos: Vec2,
    v: Vec2,
    stage: &shape::Compound,
    depth: usize,
) -> CollideAndSlideOutput {
    const MAX_BOUNCES: usize = 5;

    if depth >= MAX_BOUNCES {
        return CollideAndSlideOutput {
            final_velocity: Vec2::ZERO,
            is_grounded: false,
        };
    }

    let hit = cast_shapes(
        &Pose::new(pos, 0.0),
        v,
        &shape::Ball::new(PLAYER_SIZE),
        &Pose::IDENTITY,
        Vec2::ZERO,
        stage,
        ShapeCastOptions::default(),
    )
    .unwrap();

    let Some(hit) = hit else {
        return CollideAndSlideOutput {
            final_velocity: v,
            is_grounded: false,
        };
    };

    if !(0.0 < hit.time_of_impact && hit.time_of_impact <= 1.0) {
        return CollideAndSlideOutput {
            final_velocity: v,
            is_grounded: false,
        };
    }

    let mut snap_to_surface =
        v.normalize_or_zero() * (v.length() * hit.time_of_impact - SKIN_WIDTH);

    let rem_v = v - snap_to_surface;

    if snap_to_surface.length() <= SKIN_WIDTH {
        snap_to_surface = Vec2::ZERO;
    }

    let (rem_v, is_grounded) = slide(rem_v, hit.normal2);
    let output = collide_and_slide(player_shape, pos + snap_to_surface, rem_v, stage, depth + 1);

    CollideAndSlideOutput {
        final_velocity: snap_to_surface + output.final_velocity,
        is_grounded: is_grounded || output.is_grounded,
    }
}

pub fn slide(mut v: Vec2, normal: Vec2) -> (Vec2, bool) {
    let mut is_grounded = false;

    let stage_angle = normal.to_angle();
    if MIN_GROUNDING_ANGLE <= stage_angle && stage_angle <= MAX_GROUNDING_ANGLE {
        v.y = v.x * (-normal.x / normal.y);
        is_grounded = true;
    } else if MIN_UPWARD_BLOCKING_ANGLE <= stage_angle && stage_angle <= MAX_UPWARD_BLOCKING_ANGLE {
        v.y = v.y.min(0.0);
    } else {
        let vx_snap_to_surface = v.y / (-normal.x / normal.y);
        if normal.x < 0.0 {
            v.x = v.x.min(vx_snap_to_surface);
        } else {
            v.x = v.x.max(vx_snap_to_surface);
        }
    }

    return (v, is_grounded);
}
