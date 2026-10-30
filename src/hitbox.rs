use parry2d::math::Vector2;

pub struct Animation {
    pub frames: Vec<Frame>,
}

pub struct Frame {
    /// Arranged by chronological priority from highest to lowest
    pub hitboxes: Vec<HitboxGroup>,
    pub texture: (),
}

pub struct HitboxGroup {
    /// Used for hitbox interpolation between frames, and to makes sure the opponent doens't get hit more than once
    id: u8,
    /// Arranged from highest to lowest priority
    hitboxes: Vec<Hitbox>,
}

pub struct Hitbox {
    pub radius: f64,
    pub location: Vector2,
    pub effect: HitboxEffect,
}

pub struct HitboxEffect {
    /// (Is expressed in radians)
    launch_angle: f64,
    knockback: Scaling,
    hitstun: Scaling,
}

pub struct Scaling {
    initial: f64,
    bonus: f64,
}
