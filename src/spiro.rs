use crate::{dragging::Draggable, prelude::*};

#[derive(Component)]
struct Fixed;

#[derive(Component)]
struct Rotation(f32);

#[derive(Component)]
pub struct Speed(pub f32);

#[derive(Component)]
struct Gear;

#[derive(Component)]
pub struct Radius(pub f32);

#[derive(Component)]
pub struct Pen(pub f32);

#[derive(Component)]
struct GearColor(Srgba);

#[derive(Component)]
pub struct LineColor(pub Srgba);

#[derive(Component)]
pub struct Line(pub Vec<Vec2>);

#[derive(Component)]
pub struct Paused;

#[derive(Component)]
struct PenPos(Vec2);

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
struct FixedGearBundle {
    gear: Gear,
    radius: Radius,
    gear_color: GearColor,
    spatial_bundle: SpatialBundle,
    fixed: Fixed,
    draggable: Draggable,
}

impl Default for FixedGearBundle {
    fn default() -> Self {
        Self {
            gear: Gear,
            fixed: Fixed,
            draggable: Draggable,
            radius: Radius(75.0),
            gear_color: GearColor(color::AMBER_600),
            spatial_bundle: SpatialBundle::default(),
        }
    }
}

#[derive(Bundle)]
pub struct RotatingGearBundle {
    gear: Gear,
    radius: Radius,
    gear_color: GearColor,
    spatial_bundle: SpatialBundle,
    rotation: Rotation,
    speed: Speed,
    pen: Pen,
    pen_pos: PenPos,
    line: Line,
    line_color: LineColor,
}

impl Default for RotatingGearBundle {
    fn default() -> Self {
        Self {
            gear: Gear,
            rotation: Rotation(0.0),
            speed: Speed(0.1),
            radius: Radius(0.0),
            pen: Pen(20.0),
            pen_pos: PenPos(Vec2::ZERO),
            gear_color: GearColor(color::PURPLE_600),
            line: Line(Vec::new()),
            line_color: LineColor(color::EMERALD_600),
            spatial_bundle: SpatialBundle::default(),
        }
    }
}

fn setup(mut commands: Commands) {
    commands.insert_resource(Settings::default());

    let fixed_gear = FixedGearBundle::default();

    let gear_2_radius = 27.5;
    let mut rotating_gear = RotatingGearBundle::default();
    rotating_gear.radius = Radius(gear_2_radius);
    rotating_gear.spatial_bundle.transform.translation.y =
        fixed_gear.spatial_bundle.transform.translation.y + (fixed_gear.radius.0 + gear_2_radius);

    // TODO: Each rotating gear needs to be attached to a fixed gear
    commands.spawn(rotating_gear);
    commands.spawn(fixed_gear);
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

fn update_line(mut q_pen_pos: Query<(&mut Line, &PenPos)>) {
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
    q_fixed: Query<(&Transform, &Radius), (With<Fixed>, Without<Rotation>)>,
    mut q_gears: Query<
        (&mut Transform, &mut Rotation, &Speed, &Radius),
        (With<Rotation>, Without<Fixed>, Without<Paused>),
    >,
) {
    let (fixed_transform, &Radius(fixed_radius)) = r!(q_fixed.get_single());

    for (mut rotating_transform, mut rotation, Speed(speed), &Radius(rotating_radius)) in
        q_gears.iter_mut()
    {
        // Move the rotating gear around the fixed gear
        rotation.0 += speed;

        // Based on the rotation, calculate the new position and the new angle of the rotating gea,
        let (angle, new_pos) = new_angle_and_center(rotation.0, fixed_radius, rotating_radius);

        rotating_transform.translation = fixed_transform.translation + new_pos.extend(0.0);
        rotating_transform.rotation = Quat::from_rotation_z(-angle);
    }
}
