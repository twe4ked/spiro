use crate::{
    prelude::*,
    spiro::{
        Fixed, FixedGearBundle, GearColor, Line, LineColor, Paused, Pen, Radius,
        RotatingGearBundle, Rotation, Settings, Speed,
    },
};
use bevy_egui::{
    egui::{self, Button, Color32, CursorIcon, DragValue, Frame, Grid, ScrollArea, SidePanel, Ui},
    EguiContexts,
};

#[derive(Resource)]
pub struct Cursor(pub Option<CursorIcon>);

pub(super) fn plugin(app: &mut App) {
    app //
        .insert_resource(Cursor(None))
        .add_systems(Update, (input, ui, update_cursor_icon));
}

fn input(mut settings: ResMut<Settings>, keys: Res<ButtonInput<KeyCode>>) {
    if keys.just_pressed(KeyCode::Escape) {
        settings.show_sidebar = !settings.show_sidebar;
    }
}

fn ui(
    mut commands: Commands,
    mut contexts: EguiContexts,
    mut q_fixed: Query<
        (Entity, &mut Radius, &mut GearColor, &Children),
        (With<Fixed>, Without<Rotation>),
    >,
    mut q_rotating: Query<
        (
            Entity,
            &mut Line,
            &mut LineColor,
            &mut GearColor,
            &mut Speed,
            &mut Pen,
            &mut Radius,
            Option<&Paused>,
        ),
        (With<Rotation>, Without<Fixed>),
    >,
    mut settings: ResMut<Settings>,
) {
    SidePanel::left("SPIRO")
        .resizable(false)
        .frame(Frame::none().fill(Color32::BLACK).inner_margin(10.0))
        .show_animated(contexts.ctx_mut(), settings.show_sidebar, |ui| {
            ScrollArea::vertical().show(ui, |ui| {
                for (i_fixed, (fixed_entity, mut radius, mut gear_color, children)) in
                    q_fixed.iter_mut().enumerate()
                {
                    // Fixed gear
                    Grid::new(format!("grid: {i_fixed}"))
                        .num_columns(2)
                        .spacing([40.0, 4.0])
                        .striped(true)
                        .show(ui, |mut ui| {
                            ui.label("Radius");
                            ui.add(DragValue::new(&mut radius.0).range(1.0..=128.0).speed(0.1));
                            ui.end_row();

                            ui.label("Gear color");
                            color_picker(&mut ui, &mut gear_color.0);
                            ui.end_row();
                        });
                    ui.separator();

                    // Rotating gears
                    for (i, child) in children.iter().enumerate() {
                        if let Ok((
                            rotating_entity,
                            mut line,
                            mut line_color,
                            mut gear_color,
                            mut speed,
                            mut pen,
                            mut radius,
                            paused,
                        )) = q_rotating.get_mut(*child)
                        {
                            // Gear settings
                            Grid::new(format!("grid {i_fixed} {i}"))
                                .num_columns(2)
                                .spacing([40.0, 4.0])
                                .striped(true)
                                .show(ui, |mut ui| {
                                    ui.label("Speed");
                                    ui.add(
                                        DragValue::new(&mut speed.0).range(0.01..=1.0).speed(0.01),
                                    );
                                    ui.end_row();

                                    ui.label("Radius");
                                    ui.add(
                                        DragValue::new(&mut radius.0).range(1.0..=128.0).speed(0.1),
                                    );
                                    ui.end_row();

                                    ui.label("Pen distance");
                                    ui.add(
                                        DragValue::new(&mut pen.0).range(1.0..=128.0).speed(0.1),
                                    );
                                    ui.end_row();

                                    ui.label("Line color");
                                    color_picker(&mut ui, &mut line_color.0);
                                    ui.end_row();

                                    ui.label("Gear color");
                                    color_picker(&mut ui, &mut gear_color.0);
                                    ui.end_row();
                                });

                            // Gear controls
                            ui.horizontal(|ui| {
                                if ui.add(Button::new("Clear line")).clicked() {
                                    line.0 = Vec::new();
                                }

                                if ui.add(Button::new("Remove gear")).clicked() {
                                    commands.entity(rotating_entity).despawn();
                                }

                                {
                                    let mut toggle = paused.is_some();
                                    ui.toggle_value(&mut toggle, "Pause");
                                    if toggle != paused.is_some() {
                                        if toggle {
                                            commands.entity(rotating_entity).insert(Paused);
                                        } else {
                                            commands.entity(rotating_entity).remove::<Paused>();
                                        }
                                    }
                                }
                            });

                            ui.separator();
                        }
                    }

                    // Spirograph controls
                    ui.horizontal(|ui| {
                        if ui.add(Button::new("Add gear")).clicked() {
                            commands.entity(fixed_entity).with_children(|parent| {
                                parent.spawn(RotatingGearBundle::default());
                            });
                        }

                        if ui.add(Button::new("Remove spirograph")).clicked() {
                            commands.entity(fixed_entity).despawn_recursive();
                        }
                    });

                    ui.separator();
                }

                // Global controls
                ui.horizontal(|ui| {
                    ui.toggle_value(&mut settings.gizmos_enabled, "Enable gizmos");

                    if ui.add(Button::new("Clear all")).clicked() {
                        for (_entity, mut line, ..) in q_rotating.iter_mut() {
                            line.0 = Vec::new();
                        }
                    }

                    if ui.add(Button::new("Add")).clicked() {
                        commands
                            .spawn(FixedGearBundle::new(Vec3::ZERO.with_x(200.0)))
                            .with_children(|parent| {
                                parent.spawn(RotatingGearBundle::default());
                            });
                    }
                });

                ui.separator();

                ui.label("Hit escape to toggle sidebar");
            });
        });
}

fn update_cursor_icon(mut contexts: EguiContexts, cursor: Res<Cursor>) {
    if let Some(cursor_icon) = cursor.0 {
        let ctx = contexts.ctx_mut();
        ctx.set_cursor_icon(cursor_icon);
    }
}

fn color_picker(mut ui: &mut Ui, line_color: &mut Srgba) {
    let [r, g, b, a] = line_color.to_f32_array();
    let mut egui_color: egui::Rgba = egui::Rgba::from_srgba_unmultiplied(
        (r * 255.0) as u8,
        (g * 255.0) as u8,
        (b * 255.0) as u8,
        (a * 255.0) as u8,
    );

    egui::widgets::color_picker::color_edit_button_rgba(
        &mut ui,
        &mut egui_color,
        egui::color_picker::Alpha::Opaque,
    );

    let [r, g, b, a] = egui_color.to_srgba_unmultiplied();
    *line_color = Color::srgba(
        r as f32 / 255.0,
        g as f32 / 255.0,
        b as f32 / 255.0,
        a as f32 / 255.0,
    )
    .into();
}
