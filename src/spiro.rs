use crate::{dragging::Draggable, prelude::*};
use bevy_egui::{egui, EguiContexts};

#[derive(Component)]
struct Fixed;

#[derive(Component)]
struct Rotation(f32);

#[derive(Component)]
struct Speed(f32);

#[derive(Component)]
struct Gear;

#[derive(Component)]
pub struct Radius(pub f32);

#[derive(Component)]
struct Pen(f32);

#[derive(Component)]
struct GearColor(Srgba);

#[derive(Component)]
struct LineColor(Srgba);

#[derive(Component)]
struct Line(Vec<Vec2>);

#[derive(Component)]
struct Paused;

#[derive(Component)]
struct PenPos(Vec2);

#[derive(Resource)]
struct Settings {
    gizmos_enabled: bool,
}

impl Default for Settings {
    fn default() -> Self {
        Self {
            gizmos_enabled: true,
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
        .add_systems(Update, ui)
        .add_systems(Startup, setup);
}

fn setup(mut commands: Commands) {
    commands.insert_resource(Settings::default());

    let fixed_gear_tranlation = Vec3::new(0.0, 20.0, 0.0);

    // TODO: Each rotating gear needs to be attached to a fixed gear
    let gear_1_radius = 75.0;
    commands.spawn((
        Gear,
        Fixed,
        Radius(gear_1_radius),
        GearColor(color::AMBER_600),
        Draggable,
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
        Rotation(0.0),
        Speed(0.1),
        Radius(gear_2_radius),
        Pen(20.0),
        PenPos(Vec2::ZERO),
        GearColor(color::PURPLE_600),
        Line(Vec::new()),
        LineColor(color::EMERALD_600),
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
    q_gears: Query<(&Transform, &Radius, &PenPos, &GearColor), With<Gear>>,
    settings: Res<Settings>,
) {
    for (transform, Radius(radius), &PenPos(pen_pos), GearColor(color)) in &q_gears {
        if settings.gizmos_enabled {
            gizmos.circle_2d(transform.translation.xy(), *radius, *color);
            gizmos.circle_2d(transform.translation.xy(), 0.1, color::RED_600);
            gizmos.circle_2d(pen_pos, 1.0, color::PINK_600);
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

fn ui(
    mut commands: Commands,
    mut contexts: EguiContexts,
    mut rotating: Query<(
        Entity,
        &mut Line,
        &mut LineColor,
        &mut Speed,
        &mut Pen,
        &mut Radius,
        Option<&Paused>,
    )>,
    mut settings: ResMut<Settings>,
) {
    egui::SidePanel::left("SPIRO")
        .resizable(false)
        .frame(
            egui::Frame::none()
                .fill(egui::Color32::BLACK)
                .inner_margin(10.0),
        )
        .show(contexts.ctx_mut(), |ui| {
            for (i, (entity, mut line, mut line_color, mut speed, mut pen, mut radius, paused)) in
                rotating.iter_mut().enumerate()
            {
                egui::Grid::new(format!("grid {}", i))
                    .num_columns(2)
                    .spacing([40.0, 4.0])
                    .striped(true)
                    .show(ui, |ui| {
                        ui.label("Speed");
                        ui.add(
                            egui::DragValue::new(&mut speed.0)
                                .range(0.01..=1.0)
                                .speed(0.01),
                        );
                        ui.end_row();

                        ui.label("Radius");
                        ui.add(
                            egui::DragValue::new(&mut radius.0)
                                .range(1.0..=128.0)
                                .speed(0.1),
                        );
                        ui.end_row();

                        ui.label("Pen distance");
                        ui.add(
                            egui::DragValue::new(&mut pen.0)
                                .range(1.0..=128.0)
                                .speed(0.1),
                        );
                        ui.end_row();

                        {
                            let [r, g, b, a] = Srgba::from(line_color.0).to_f32_array();
                            let mut egui_color: egui::Rgba = egui::Rgba::from_srgba_unmultiplied(
                                (r * 255.0) as u8,
                                (g * 255.0) as u8,
                                (b * 255.0) as u8,
                                (a * 255.0) as u8,
                            );

                            ui.label("Line color");
                            egui::widgets::color_picker::color_edit_button_rgba(
                                ui,
                                &mut egui_color,
                                egui::color_picker::Alpha::Opaque,
                            );
                            ui.end_row();

                            let [r, g, b, a] = egui_color.to_srgba_unmultiplied();
                            line_color.0 = Color::srgba(
                                r as f32 / 255.0,
                                g as f32 / 255.0,
                                b as f32 / 255.0,
                                a as f32 / 255.0,
                            )
                            .into();
                        }
                    });

                if ui.add(egui::Button::new("Clear line")).clicked() {
                    line.0 = Vec::new();
                }

                if ui.add(egui::Button::new("Remove")).clicked() {
                    commands.entity(entity).despawn();
                }

                {
                    let mut toggle = paused.is_some();
                    ui.toggle_value(&mut toggle, "Pause");
                    if toggle != paused.is_some() {
                        if toggle {
                            commands.entity(entity).insert(Paused);
                        } else {
                            commands.entity(entity).remove::<Paused>();
                        }
                    }
                }

                ui.separator();
            }

            ui.toggle_value(&mut settings.gizmos_enabled, "Enable gizmos");

            if ui.add(egui::Button::new("Clear all")).clicked() {
                for (_entity, mut line, _, _, _, _, _) in rotating.iter_mut() {
                    line.0 = Vec::new();
                }
            }

            if ui.add(egui::Button::new("Add")).clicked() {
                commands.spawn((
                    Gear,
                    Rotation(90.0_f32.to_radians()),
                    Speed(0.1),
                    Radius(20.0),
                    Pen(20.0),
                    PenPos(Vec2::ZERO),
                    GearColor(color::PURPLE_600),
                    Line(Vec::new()),
                    LineColor(color::EMERALD_600),
                    SpatialBundle::default(),
                ));
            }
        });
}
