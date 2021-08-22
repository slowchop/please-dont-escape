use bevy::prelude::*;
use bevy::app::{Events, AppExit};

pub fn input_exit_system(
    mut app_exit_events: ResMut<Events<bevy::app::AppExit>>,
    keys: Res<Input<KeyCode>>,
) {
    if keys.just_pressed(KeyCode::Escape) {
        app_exit_events.send(AppExit);
    }
}
