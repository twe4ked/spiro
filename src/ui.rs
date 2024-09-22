use crate::{
    prelude::*,
    spiro::{Line, LineColor, Paused, Pen, Radius, RotatingGearBundle, Settings, Speed},
};
use bevy_egui::{egui, EguiContexts};

pub(super) fn plugin(app: &mut App) {
    app.add_systems(Update, ui);
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
            egui::ScrollArea::vertical().show(ui, |ui| {
                for (
                    i,
                    (entity, mut line, mut line_color, mut speed, mut pen, mut radius, paused),
                ) in rotating.iter_mut().enumerate()
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
                        for (_entity, mut line, _, _, _, _, _) in rotating.iter_mut() {
                            line.0 = Vec::new();
                        }
                    }

                    if ui.add(egui::Button::new("Add")).clicked() {
                        commands.spawn(RotatingGearBundle::default());
                    }
                });
            });
        });
}
