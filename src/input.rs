use bevy::app::{AppExit, Events};
use bevy::prelude::*;

pub fn exit_on_escape_key(
    mut app_exit_events: ResMut<Events<bevy::app::AppExit>>,
    keys: Res<Input<KeyCode>>,
) {
    if keys.just_pressed(KeyCode::Escape) {
        app_exit_events.send(AppExit);
    }
}
