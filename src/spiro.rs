use crate::{
    dragging::{DragEnd, DragStart, Draggable},
    prelude::*,
};
use rand::Rng;
use std::f32::consts::TAU;

#[derive(Component)]
pub struct Fixed;

#[derive(Component)]
pub struct Rotation(f32);

#[derive(Component)]
pub struct Speed(pub f32);

#[derive(Component)]
pub struct Gear;

#[derive(Component)]
pub struct Radius(pub f32);

#[derive(Component)]
pub struct Pen(pub f32);

#[derive(Component)]
pub struct GearColor(pub Srgba);

#[derive(Component)]
pub struct LineColor(pub Srgba);

#[derive(Component)]
pub struct Line(pub Vec<Vec2>);

#[derive(Component)]
pub struct Paused;

#[derive(Component)]
pub struct PenPos(Vec2);

#[derive(Resource)]
pub struct Settings {
    pub gizmos_enabled: bool,
    pub show_sidebar: bool,
}

impl Default for Settings {
    fn default() -> Self {
        Self {
            gizmos_enabled: true,
            show_sidebar: true,
        }
    }
}

const RAINBOW: [Srgba; 17] = [
    color::RED_600,
    color::ORANGE_600,
    color::AMBER_600,
    color::YELLOW_600,
    color::LIME_600,
    color::GREEN_600,
    color::EMERALD_600,
    color::TEAL_600,
    color::CYAN_600,
    color::SKY_600,
    color::BLUE_600,
    color::INDIGO_600,
    color::VIOLET_600,
    color::PURPLE_600,
    color::FUCHSIA_600,
    color::PINK_600,
    color::ROSE_600,
];

pub(super) fn plugin(app: &mut App) {
    app //
        .observe(drag_start)
        .observe(drag_end)
        .add_systems(
            FixedUpdate,
            (
                rotate_gears,
                update_pen_pos,
                update_line,
                draw_line,
                draw_gizmos,
            )
                .chain(),
        )
        .add_systems(Startup, setup);
}

#[derive(Bundle)]
pub struct FixedGearBundle {
    pub gear: Gear,
    // TODO: Different fixed shapes
    pub radius: Radius,
    pub gear_color: GearColor,
    pub transform_bundle: TransformBundle,
    pub fixed: Fixed,
    pub draggable: Draggable,
}

impl Default for FixedGearBundle {
    fn default() -> Self {
        Self {
            gear: Gear,
            fixed: Fixed,
            draggable: Draggable,
            radius: Radius(150.0),
            gear_color: GearColor(color::AMBER_600),
            transform_bundle: TransformBundle::default(),
        }
    }
}

impl FixedGearBundle {
    pub fn rand(bounds: Vec2) -> Self {
        let mut rng = rand::thread_rng();

        let half_extents = 0.5 * bounds;
        let translation = Vec3::new(
            rng.gen_range(-half_extents.x..half_extents.x),
            rng.gen_range(-half_extents.y..half_extents.y),
            0.0,
        );
        let radius = rng.gen_range(1.0..256.0);
        let gear_color = RAINBOW[rng.gen_range(0..RAINBOW.len())];

        Self {
            transform_bundle: TransformBundle {
                local: Transform::from_translation(translation),
                ..default()
            },
            radius: Radius(radius),
            gear_color: GearColor(gear_color),
            ..default()
        }
    }
}

#[derive(Bundle)]
pub struct RotatingGearBundle {
    pub gear: Gear,
    pub radius: Radius,
    pub gear_color: GearColor,
    pub transform_bundle: TransformBundle,
    pub rotation: Rotation,
    pub speed: Speed,
    pub pen: Pen,
    pub pen_pos: PenPos,
    pub line: Line,
    pub line_color: LineColor,
}

impl Default for RotatingGearBundle {
    fn default() -> Self {
        Self {
            gear: Gear,
            rotation: Rotation(0.0),
            speed: Speed(8.0),
            radius: Radius(55.0),
            pen: Pen(40.0),
            pen_pos: PenPos(Vec2::ZERO),
            gear_color: GearColor(color::PURPLE_600),
            line: Line(Vec::new()),
            line_color: LineColor(Srgba::BLACK),
            transform_bundle: TransformBundle::default(),
        }
    }
}

impl RotatingGearBundle {
    pub fn rand() -> Self {
        let mut rng = rand::thread_rng();

        let rotation = rng.gen_range(0.0..TAU);
        let speed = rng.gen_range(0.1..16.0);
        let radius = rng.gen_range(1.0..128.0);
        let gear_color = RAINBOW[rng.gen_range(0..RAINBOW.len())];
        let pen_dis = rng.gen_range(0.0..64.0);

        Self {
            rotation: Rotation(rotation),
            speed: Speed(speed),
            radius: Radius(radius),
            gear_color: GearColor(gear_color),
            pen: Pen(pen_dis),
            ..default()
        }
    }
}

fn setup(mut commands: Commands) {
    commands.insert_resource(Settings::default());

    commands
        .spawn(FixedGearBundle::default())
        .with_children(|parent| {
            parent.spawn(RotatingGearBundle::default());
        });
}

fn update_pen_pos(
    mut gizmos: Gizmos,
    mut rotating: Query<(&mut PenPos, &Transform, &Pen), With<Rotation>>,
    settings: Res<Settings>,
) {
    for (mut pen_pos, rotating_transform, &Pen(pen_dist)) in rotating.iter_mut() {
        if settings.gizmos_enabled {
            gizmos.axes_2d(*rotating_transform, 10.0);
        }

        // Calculate pen location
        let angle = rotating_transform.rotation.to_euler(EulerRot::XYZ).2;
        pen_pos.0 = rotating_transform.translation.xy()
            + Vec2::from_angle(angle + 90.0_f32.to_radians()) * pen_dist;
    }
}

fn draw_gizmos(
    mut gizmos: Gizmos,
    q_gears: Query<(&Transform, &Radius, Option<&PenPos>, &GearColor), With<Gear>>,
    settings: Res<Settings>,
) {
    for (transform, Radius(radius), pen_pos, GearColor(color)) in &q_gears {
        if settings.gizmos_enabled {
            gizmos.circle_2d(transform.translation.xy(), *radius, *color);
            gizmos.circle_2d(transform.translation.xy(), 0.1, color::RED_600);

            if let Some(&PenPos(pos)) = pen_pos {
                gizmos.circle_2d(pos, 1.0, color::PINK_600);
            }
        }
    }
}

fn update_line(mut q_pen_pos: Query<(&mut Line, &PenPos), Without<Paused>>) {
    for (mut line, &PenPos(pen_pos)) in q_pen_pos.iter_mut() {
        line.0.push(pen_pos);
    }
}

fn draw_line(mut gizmos: Gizmos, rotating: Query<(&Line, &LineColor)>) {
    for (line, &LineColor(line_color)) in &rotating {
        if line_color == Srgba::BLACK {
            gizmos.linestrip_gradient_2d(
                line.0.iter().copied().zip(
                    RAINBOW
                        .iter()
                        .copied()
                        .flat_map(|n| std::iter::repeat(n).take(4))
                        .cycle(),
                ),
            );
        } else {
            gizmos.linestrip_2d(line.0.iter().copied(), line_color);
        }
    }
}

// Calculate the new angle and center position of the rotating circle
fn new_angle_and_center(
    rotation_radians: f32,
    fixed_radius: f32,
    rotating_radius: f32,
) -> (f32, Vec2) {
    // Calculate the distance traveled by the center of the small circle
    let distance_traveled = rotation_radians * rotating_radius;

    // The angle through which the small circle rotates around the center of the large circle
    let angle_large_circle = distance_traveled / fixed_radius;

    // Total angle in radians for the small circle (due to rotation and rolling)
    let total_angle_small_circle = rotation_radians + angle_large_circle;

    // Calculate the new position of the center of the small circle
    let center = (fixed_radius - rotating_radius) * Vec2::from_angle(angle_large_circle);

    (total_angle_small_circle, center)
}

fn rotate_gears(
    q_fixed: Query<(&Transform, &Radius, &Children), (With<Fixed>, Without<Rotation>)>,
    mut q_gears: Query<
        (&mut Transform, &mut Rotation, &Speed, &Radius),
        (With<Rotation>, Without<Fixed>, Without<Paused>),
    >,
    time: Res<Time>,
) {
    for (fixed_transform, &Radius(fixed_radius), children) in &q_fixed {
        for &child in children.iter() {
            if let Ok((
                mut rotating_transform,
                mut rotation,
                Speed(speed),
                &Radius(rotating_radius),
            )) = q_gears.get_mut(child)
            {
                // Move the rotating gear around the fixed gear
                rotation.0 += speed * time.delta().as_secs_f32();

                // Based on the rotation, calculate the new position and the new angle of the rotating gea,
                let (angle, new_pos) =
                    new_angle_and_center(rotation.0, fixed_radius, rotating_radius);

                rotating_transform.translation = fixed_transform.translation + new_pos.extend(0.0);
                rotating_transform.rotation = Quat::from_rotation_z(-angle);
            }
        }
    }
}

fn drag_start(
    trigger: Trigger<DragStart>,
    mut commands: Commands,
    q_fixed: Query<&Children, With<Fixed>>,
    q_rotating: Query<Entity, With<Rotation>>,
) {
    for children in q_fixed.get(trigger.entity()).iter() {
        for &child in children.iter() {
            if let Ok(entity) = q_rotating.get(child) {
                commands.entity(entity).insert(Paused);
            }
        }
    }
}

fn drag_end(
    trigger: Trigger<DragEnd>,
    mut commands: Commands,
    mut q_fixed: Query<(Entity, &mut Transform, &Children), With<Fixed>>,
    q_rotating: Query<Entity, With<Rotation>>,
) {
    // A drag just finished, snap!
    const SNAP_DIST: f32 = 10.0;

    // Find the transform of the given entity
    let (_entity, t1, _children) = r!(q_fixed.get(trigger.entity()));

    // Find the closest fixed transform less than the SNAP_DIST
    let mut min_dist = f32::INFINITY;
    let mut translation = None;
    for (entity, t2, _children) in &q_fixed {
        let dist = t1.translation.distance(t2.translation);
        if entity != trigger.entity() && dist < min_dist && dist < SNAP_DIST {
            min_dist = dist;
            translation = Some(t2.translation);
        }
    }

    if let Some(translation) = translation {
        let (_entity, mut t1, _children) = r!(q_fixed.get_mut(trigger.entity()));
        t1.translation = translation;
    }

    for (_entity, _transform, children) in q_fixed.get(trigger.entity()).iter() {
        for &child in children.iter() {
            if let Ok(entity) = q_rotating.get(child) {
                commands.entity(entity).remove::<Paused>();
            }
        }
    }
}
