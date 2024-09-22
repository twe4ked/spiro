use crate::{
    prelude::*,
    spiro::{
        Fixed, Line, LineColor, Paused, Pen, Radius, RotatingGearBundle, Rotation, Settings, Speed,
    },
};
use bevy_egui::egui::CursorIcon;
use bevy_egui::{egui, EguiContexts};

#[derive(Resource)]
pub struct Cursor(pub Option<CursorIcon>);

pub(super) fn plugin(app: &mut App) {
    app //
        .insert_resource(Cursor(None))
        .add_systems(Update, (ui, update_cursor_icon));
}

fn ui(
    mut commands: Commands,
    mut contexts: EguiContexts,
    mut q_fixed: Query<&mut Radius, (With<Fixed>, Without<Rotation>)>,
    mut q_rotating: Query<
        (
            Entity,
            &mut Line,
            &mut LineColor,
            &mut Speed,
            &mut Pen,
            &mut Radius,
            Option<&Paused>,
        ),
        (With<Rotation>, Without<Fixed>),
    >,
    mut settings: ResMut<Settings>,
    cursor: Res<Cursor>,
    keys: Res<ButtonInput<KeyCode>>,
) {
    if keys.just_pressed(KeyCode::Escape) {
        settings.show_sidebar = !settings.show_sidebar;
    }

    if let Some(cursor_icon) = cursor.0 {
        let ctx = contexts.ctx_mut();
        ctx.set_cursor_icon(cursor_icon);
    }

    egui::SidePanel::left("SPIRO")
        .resizable(false)
        .frame(
            egui::Frame::none()
                .fill(egui::Color32::BLACK)
                .inner_margin(10.0),
        )
        .show_animated(contexts.ctx_mut(), settings.show_sidebar, |ui| {
            egui::ScrollArea::vertical().show(ui, |ui| {
                // Fixed gear
                let mut radius = r!(q_fixed.get_single_mut());
                egui::Grid::new(format!("grid"))
                    .num_columns(2)
                    .spacing([40.0, 4.0])
                    .striped(true)
                    .show(ui, |ui| {
                        ui.label("Radius");
                        ui.add(
                            egui::DragValue::new(&mut radius.0)
                                .range(1.0..=128.0)
                                .speed(0.1),
                        );
                        ui.end_row();
                    });
                ui.separator();

                // Rotating gears
                for (
                    i,
                    (entity, mut line, mut line_color, mut speed, mut pen, mut radius, paused),
                ) in q_rotating.iter_mut().enumerate()
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
                                let mut egui_color: egui::Rgba =
                                    egui::Rgba::from_srgba_unmultiplied(
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
                    ui.horizontal(|ui| {
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
                    });

                    ui.separator();
                }

                ui.horizontal(|ui| {
                    ui.toggle_value(&mut settings.gizmos_enabled, "Enable gizmos");

                    if ui.add(egui::Button::new("Clear all")).clicked() {
                        for (_entity, mut line, ..) in q_rotating.iter_mut() {
                            line.0 = Vec::new();
                        }
                    }

                    if ui.add(egui::Button::new("Add")).clicked() {
                        commands.spawn(RotatingGearBundle::default());
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
