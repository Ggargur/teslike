//! Kinematic character movement shared by the player and the enemies:
//! acceleration toward a wish velocity, gravity, knockback, steps and
//! ground snapping, on top of avian's move-and-slide.

use avian3d::prelude::*;
use bevy::prelude::*;

#[derive(PhysicsLayer, Clone, Copy, Debug, Default)]
pub enum Layer {
    #[default]
    World,
    Character,
    Trigger,
}

#[derive(Component, Debug, Reflect)]
pub struct Mover {
    /// Desired horizontal velocity.
    pub wish: Vec3,
    pub accel: f32,
    pub decel: f32,
    pub gravity: f32,
    pub step_height: f32,
    /// Horizontal + vertical velocity from walking and falling.
    pub velocity: Vec3,
    /// External push (kicks, hits). Decays on its own.
    pub knockback: Vec3,
    pub knockback_friction: f32,
    pub grounded: bool,
}

impl Default for Mover {
    fn default() -> Self {
        Self {
            wish: Vec3::ZERO,
            accel: 30.0,
            decel: 25.0,
            gravity: 20.0,
            step_height: 0.3,
            velocity: Vec3::ZERO,
            knockback: Vec3::ZERO,
            knockback_friction: 6.0,
            grounded: false,
        }
    }
}

impl Mover {
    /// Total velocity the body moved with last frame.
    pub fn total(&self) -> Vec3 {
        self.velocity + self.knockback
    }
}

/// Mask used for world/character collision queries (no triggers).
pub fn solid_filter(exclude: Entity) -> SpatialQueryFilter {
    SpatialQueryFilter::from_mask([Layer::World, Layer::Character]).with_excluded_entities([exclude])
}

pub struct MoverPlugin;

impl Plugin for MoverPlugin {
    fn build(&self, app: &mut App) {
        app.register_type::<Mover>().add_systems(Update, move_bodies.in_set(MoveSet));
    }
}

#[derive(SystemSet, Debug, Clone, PartialEq, Eq, Hash)]
pub struct MoveSet;

/// Distance down to ground whose normal is at least `min_normal_y` upright.
fn ground_hit(q: &SpatialQuery, col: &Collider, pos: Vec3, dist: f32, min_normal_y: f32, filter: &SpatialQueryFilter) -> Option<f32> {
    let cfg = ShapeCastConfig { ignore_origin_penetration: true, ..ShapeCastConfig::from_max_distance(dist) };
    q.cast_shape(col, pos, Quat::IDENTITY, Dir3::NEG_Y, &cfg, filter)
        .filter(|h| h.normal1.y > min_normal_y)
        .map(|h| h.distance)
}

fn move_bodies(
    time: Res<Time>,
    q: SpatialQuery,
    mas: MoveAndSlide,
    mut bodies: Query<(Entity, &mut Transform, &mut Mover, &Collider), Without<RigidBodyDisabled>>,
) {
    let dt = time.delta_secs();
    if dt <= 0.0 {
        return;
    }
    let cfg = MoveAndSlideConfig::default();
    for (e, mut tf, mut m, col) in &mut bodies {
        let filter = solid_filter(e);
        // Walk acceleration on the horizontal plane.
        let horiz = Vec3::new(m.velocity.x, 0.0, m.velocity.z);
        let rate = if m.wish.length_squared() > horiz.length_squared() { m.accel } else { m.decel };
        let horiz = horiz.move_towards(m.wish, rate * dt);
        let vy = if m.grounded { -1.0 } else { m.velocity.y - m.gravity * dt };
        m.velocity = Vec3::new(horiz.x, vy, horiz.z);
        let friction = if m.grounded { m.knockback_friction } else { m.knockback_friction * 0.15 };
        let kb = m.knockback * (-friction * dt).exp();
        m.knockback = Vec3::new(kb.x, 0.0, kb.z);

        let total = m.velocity + m.knockback;
        let start = tf.translation;
        let out = mas.move_and_slide(col, start, Quat::IDENTITY, total, time.delta(), &cfg, &filter, |_| {
            MoveAndSlideHitResponse::Accept
        });
        let mut pos = out.position;

        // Step up: if blocked horizontally while grounded, retry from higher up.
        let wanted = Vec2::new(total.x, total.z) * dt;
        let got = Vec2::new(pos.x - start.x, pos.z - start.z);
        if m.grounded && wanted.length() > 1e-3 && got.length() < wanted.length() * 0.5 {
            let up = ShapeCastConfig { ignore_origin_penetration: true, ..ShapeCastConfig::from_max_distance(m.step_height) };
            let clear = q.cast_shape(col, start, Quat::IDENTITY, Dir3::Y, &up, &filter).is_none();
            if clear {
                let raised = start + Vec3::Y * m.step_height;
                let flat = Vec3::new(total.x, 0.0, total.z);
                let o2 = mas.move_and_slide(col, raised, Quat::IDENTITY, flat, time.delta(), &cfg, &filter, |_| {
                    MoveAndSlideHitResponse::Accept
                });
                let got2 = Vec2::new(o2.position.x - start.x, o2.position.z - start.z);
                if got2.length() > got.length() + 1e-3
                    && let Some(d) = ground_hit(&q, col, o2.position, m.step_height, 0.05, &filter)
                {
                    pos = o2.position - Vec3::Y * d;
                }
            }
        }

        // Ground check and snap.
        let was_grounded = m.grounded;
        match ground_hit(&q, col, pos, if was_grounded { 0.25 } else { 0.05 }, 0.3, &filter) {
            Some(d) if m.velocity.y <= 0.0 => {
                pos.y -= d;
                m.grounded = true;
                m.velocity.y = 0.0;
            }
            _ => m.grounded = false,
        }
        tf.translation = pos;
    }
}
