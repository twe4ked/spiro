#![allow(clippy::type_complexity)]

use bevy_debug_text_overlay::OverlayPlugin as DebugTextPlugin;
use bevy_egui::EguiPlugin;

mod dragging;
mod spiro;
mod ui;

pub mod prelude {
    pub use bevy::color::palettes::tailwind as color;
    pub use bevy::prelude::*;
    pub use bevy_debug_text_overlay::screen_print;
    pub use tiny_bail::prelude::*;
}

use prelude::*;

const TIME_STEP: f64 = 1.0 / 60.0;

pub struct LibPlugin;

impl Plugin for LibPlugin {
    fn build(&self, app: &mut App) {
        let window_plugin = WindowPlugin {
            primary_window: Window {
                title: "SPIRO".to_string(),
                canvas: Some("#bevy".to_string()),
                fit_canvas_to_parent: true,
                prevent_default_event_handling: true,
                ..default()
            }
            .into(),
            ..default()
        };

        app // Bevy App
            .insert_resource(Time::<Fixed>::from_seconds(TIME_STEP))
            .insert_resource(ClearColor(Color::BLACK))
            .add_plugins(DefaultPlugins.set(window_plugin))
            .add_plugins(DebugTextPlugin {
                fallback_color: color::SLATE_50.into(),
                ..default()
            })
            .add_plugins(EguiPlugin)
            .add_plugins((
                //
                spiro::plugin,
                ui::plugin,
                dragging::plugin,
            ))
            .add_systems(Startup, spawn_camera);
    }
}

fn spawn_camera(mut commands: Commands) {
    commands.spawn(Camera2dBundle::default());
}
