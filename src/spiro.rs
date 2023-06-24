use crate::prelude::*;
use bevy_egui::{egui, EguiContexts};

#[derive(Component)]
struct Fixed;

#[derive(Component)]
struct Rotation {
    angle: f32,
    speed: f32,
}

#[derive(Component)]
struct Gear;

#[derive(Component)]
struct Radius(f32);

#[derive(Component)]
struct Pen(f32);

#[derive(Component)]
struct Color(Srgba);

#[derive(Component)]
struct Line(Vec<Vec2>);

#[derive(Resource)]
struct Settings {
    paused: bool,
    gizmos_enabled: bool,
    line_enabled: bool,
}

impl Default for Settings {
    fn default() -> Self {
        Self {
            paused: false,
            gizmos_enabled: true,
            line_enabled: true,
        }
    }
}

pub(super) fn plugin(app: &mut App) {
    app.observe(on_spawn_gears)
        .add_systems(FixedUpdate, (update, rotate_gears))
        .add_systems(Update, (render, ui))
        .add_systems(Startup, setup);
}

fn setup(mut commands: Commands) {
    commands.insert_resource(Settings::default());
    commands.trigger(SpawnGears);
    commands.spawn(Line(Vec::new()));
}

#[derive(Event, Debug)]
pub struct SpawnGears;

fn on_spawn_gears(_trigger: Trigger<SpawnGears>, mut commands: Commands) {
    let fixed_gear_tranlation = Vec3::new(0.0, 20.0, 0.0);

    // TODO: Each rotating gear needs to be attached to a fixed gear
    let gear_1_radius = 75.0;
    commands.spawn((
        Gear,
        Fixed,
        Radius(gear_1_radius),
        Color(color::AMBER_600),
        SpatialBundle {
            transform: Transform {
                translation: fixed_gear_tranlation,
                ..default()
            },
            ..default()
        },
    ));

    let gear_2_radius = 27.5;
    commands.spawn((
        Gear,
        Rotation {
            angle: 0.0,
            speed: 0.1,
        },
        Radius(gear_2_radius),
        Pen(20.0),
        Color(color::PURPLE_600),
        SpatialBundle {
            transform: Transform {
                translation: Vec3::new(
                    0.0,
                    fixed_gear_tranlation.y + (gear_1_radius + gear_2_radius),
                    0.0,
                ),
                ..default()
            },
            ..default()
        },
    ));
}

fn update(
    mut gizmos: Gizmos,
    rotating: Query<&Transform, With<Rotation>>,
    pen: Query<&Pen>,
    mut line_q: Query<&mut Line>,
    settings: Res<Settings>,
) {
    let rotating_transform = r!(rotating.get_single());
    let &Pen(pen_dist) = r!(pen.get_single());
    let mut line = r!(line_q.get_single_mut());

    if settings.gizmos_enabled {
        gizmos.axes_2d(*rotating_transform, 10.0);
    }

    // Calculate pen location
    let new_pen = {
        let angle = rotating_transform.rotation.to_euler(EulerRot::XYZ).2;
        rotating_transform.translation.xy()
            + Vec2::from_angle(angle + 90.0_f32.to_radians()) * pen_dist
    };
    // Draw pen location
    if settings.gizmos_enabled {
        gizmos.circle_2d(new_pen, 1.0, color::PINK_600);
    }

    let colors = vec![
        color::SLATE_600,
        color::GRAY_600,
        color::ZINC_600,
        color::NEUTRAL_600,
        color::STONE_600,
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

    // Save the pen location and draw the current line
    if settings.line_enabled {
        line.0.push(new_pen);
        gizmos.linestrip_gradient_2d(
            line.0.iter().copied().zip(
                colors
                    .iter()
                    .copied()
                    .flat_map(|n| std::iter::repeat(n).take(4))
                    .cycle(),
            ),
        );
    }
}

fn rotate_gears(
    fixed: Query<(&Transform, &Radius), (With<Fixed>, Without<Rotation>)>,
    mut rotating: Query<(&mut Transform, &mut Rotation, &Radius), (With<Rotation>, Without<Fixed>)>,
    settings: Res<Settings>,
) {
    let (fixed_transform, Radius(fixed_radius)) = r!(fixed.get_single());
    let (mut rotating_transform, mut rotation, Radius(rotating_radius)) =
        r!(rotating.get_single_mut());

    // Calculate the new angle and center position of the rotating circle
    let (angle, center) = {
        let rotation_radians = rotation.angle;

        // Calculate the distance traveled by the center of the small circle
        let distance_traveled = rotation_radians * rotating_radius;

        // The angle through which the small circle rotates around the center of the large circle
        let angle_large_circle = distance_traveled / fixed_radius;

        // Total angle in radians for the small circle (due to rotation and rolling)
        let total_angle_small_circle = rotation_radians + angle_large_circle;

        // Calculate the new position of the center of the small circle
        let center = (fixed_radius - rotating_radius) * Vec2::from_angle(angle_large_circle);

        (total_angle_small_circle, center)
    };

    // Move the small circle around the circle
    if !settings.paused {
        rotation.angle += rotation.speed;
    }

    rotating_transform.translation = fixed_transform.translation + center.extend(0.0);
    rotating_transform.rotation = Quat::from_rotation_z(-angle);
}

fn render(
    mut gizmos: Gizmos,
    gears: Query<(&Transform, &Radius, &Color), With<Gear>>,
    settings: Res<Settings>,
) {
    for (transform, Radius(radius), Color(color)) in &gears {
        if settings.gizmos_enabled {
            gizmos.circle_2d(transform.translation.xy(), *radius, *color);
            gizmos.circle_2d(transform.translation.xy(), 0.1, color::RED_600);
        }
    }
}

fn ui(
    mut contexts: EguiContexts,
    mut rotating: Query<(&mut Rotation, &mut Pen, &mut Radius)>,
    mut line_q: Query<&mut Line>,
    mut settings: ResMut<Settings>,
) {
    let (mut rotation, mut pen, mut radius) = r!(rotating.get_single_mut());
    let mut line = r!(line_q.get_single_mut());

    egui::SidePanel::left("SPIRO")
        .resizable(false)
        .min_width(140.0)
        .frame(
            egui::Frame::none()
                .fill(egui::Color32::BLACK)
                .inner_margin(10.0),
        )
        .show(contexts.ctx_mut(), |ui| {
            ui.label("Speed");
            ui.add(
                egui::DragValue::new(&mut rotation.speed)
                    .range(0.01..=1.0)
                    .speed(0.01),
            );

            ui.label("Radius");
            ui.add(
                egui::DragValue::new(&mut radius.0)
                    .range(1.0..=128.0)
                    .speed(0.1),
            );

            ui.label("Pen distance");
            ui.add(
                egui::DragValue::new(&mut pen.0)
                    .range(1.0..=128.0)
                    .speed(0.1),
            );

            ui.horizontal(|ui| {
                ui.label("Line");
                if ui.add(egui::Button::new("Clear")).clicked() {
                    line.0 = Vec::new();
                }
            });

            ui.checkbox(&mut settings.paused, "Pause");
            ui.checkbox(&mut settings.gizmos_enabled, "Enable gizmos");
        });
}
