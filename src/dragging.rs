use crate::prelude::*;
use crate::spiro::Radius;
use bevy::{
    input::common_conditions::{input_just_pressed, input_just_released},
    window::PrimaryWindow,
};

pub(super) fn plugin(app: &mut App) {
    app //
        .insert_resource(CursorWorldPos(None))
        .add_systems(
            Update,
            (
                get_cursor_world_pos,
                (
                    start_drag.run_if(input_just_pressed(MouseButton::Left)),
                    end_drag.run_if(input_just_released(MouseButton::Left)),
                    drag.run_if(resource_exists::<DragOperation>),
                ),
            )
                .chain(),
        );
}

/// The projected 2D world coordinates of the cursor (if it's within primary window bounds).
#[derive(Resource)]
struct CursorWorldPos(Option<Vec2>);

/// The current drag operation including the offset
#[derive(Resource)]
struct DragOperation(Vec2);

#[derive(Component)]
pub struct Draggable;

// Project the cursor into the world coordinates and store it in a resource for easy use
fn get_cursor_world_pos(
    mut cursor_world_pos: ResMut<CursorWorldPos>,
    q_primary_window: Query<&Window, With<PrimaryWindow>>,
    q_camera: Query<(&Camera, &GlobalTransform)>,
) {
    let primary_window = q_primary_window.single();
    let (main_camera, main_camera_transform) = q_camera.single();
    // Get the cursor position in the world
    cursor_world_pos.0 = primary_window
        .cursor_position()
        .and_then(|cursor_pos| main_camera.viewport_to_world_2d(main_camera_transform, cursor_pos));
}

// Start the drag operation and record the offset we started dragging from
fn start_drag(
    mut commands: Commands,
    cursor_world_pos: Res<CursorWorldPos>,
    q_draggable: Query<(&Transform, &Radius), With<Draggable>>,
) {
    // If the cursor is not within the primary window skip this system
    let Some(cursor_world_pos) = cursor_world_pos.0 else {
        return;
    };

    // Get the offset from the cursor to transform
    let (transform, &Radius(radius)) = q_draggable.single();
    let drag_offset = transform.translation.truncate() - cursor_world_pos;

    // If the cursor is within the cricle start the drag operation and remember the offset of the
    // cursor from the origin
    if drag_offset.length() < radius {
        commands.insert_resource(DragOperation(drag_offset));
    }
}

fn end_drag(mut commands: Commands) {
    commands.remove_resource::<DragOperation>();
}

fn drag(
    drag_offset: Res<DragOperation>,
    cursor_world_pos: Res<CursorWorldPos>,
    mut q_draggable: Query<&mut Transform, With<Draggable>>,
) {
    // If the cursor is not within the primary window skip this system
    let Some(cursor_world_pos) = cursor_world_pos.0 else {
        return;
    };

    // Get the current Bevy logo transform
    let mut transform = q_draggable.single_mut();

    // Calculate the new translation of the Bevy logo based on cursor and drag offset
    let new_translation = cursor_world_pos + drag_offset.0;

    // Update the translation of Bevy logo transform to new translation
    transform.translation = new_translation.extend(transform.translation.z);
}
