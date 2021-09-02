use bevy::ecs::component::Component;
use bevy::prelude::*;
use std::fmt::Debug;
use std::hash::Hash;
use std::marker::PhantomData;

pub struct SplashScreen;

#[derive(Debug)]
pub struct SplashScreenConfig {
    pub timer: Timer,
    pub image: String,
}

struct InternalState {
    handle: Handle<ColorMaterial>,
    current: SplashScreenConfig,
}

impl Plugin for SplashScreen {
    fn build(&self, app: &mut AppBuilder) {
        app.add_startup_system(setup.system());
        app.add_system(check.system());
        //
        // .add_system_set(SystemSet::on_enter(self.state.clone()).with_system(setup.system()))
        // .add_system_set(
        //     SystemSet::on_update(self.state.clone())
        //         .with_system(timer.system())
        //         .with_system(escape_key.system())
        //         .with_system(mouse_click.system()),
        // )
        // .add_system_set(SystemSet::on_exit(self.state.clone()).with_system(cleanup.system()));
    }
}

fn setup(mut commands: Commands) {
    commands.spawn_bundle(OrthographicCameraBundle::new_2d());
}

fn check(
    mut commands: Commands,
    asset_server: Res<AssetServer>,
    mut materials: ResMut<Assets<ColorMaterial>>,
    mut config: Res<SplashScreenConfig>,
    mut state: Query<&mut InternalState>,
) {
    let should_load = match state.single_mut() {
        Err(_) => true,
        Ok(s) => config.image != s.current.image,
    };

    if should_load {
        let handle = asset_server.load(config.image.as_str());
        commands.spawn_bundle(SpriteBundle {
            material: materials.add(handle.into()),
            ..Default::default()
        });
    }
}

// fn set_next_state(state: &mut ResMut<State<AppState>>) {
//     // state
//     //     .set(AppState::MainMenu)
//     //     .expect("Could not set state to MainMenu.");
// }

// fn timer(mut state: ResMut<State<AppState>>, time: Res<Time>, mut timer: ResMut<SplashTimer>) {
//     if timer.0.tick(time.delta()).just_finished() {
//         set_next_state(&mut state);
//     }
// }
//
// fn escape_key(mut state: ResMut<State<AppState>>, mut keys: ResMut<Input<KeyCode>>) {
//     if keys.just_pressed(KeyCode::Escape) {
//         // Reset so that the main menu doesn't get an escape key event immediately.
//         keys.reset(KeyCode::Escape);
//         set_next_state(&mut state);
//     }
// }
//
// fn mouse_click(mut state: ResMut<State<AppState>>, mut mouse_button: ResMut<Input<MouseButton>>) {
//     if mouse_button.just_pressed(MouseButton::Left) {
//         mouse_button.reset(MouseButton::Left);
//         set_next_state(&mut state);
//     }
// }
//
// fn cleanup(mut commands: Commands, entities: Query<Entity>) {
//     debug!("Cleanup splash.");
//     for sprite in entities.iter() {
//         commands.entity(sprite).despawn_recursive();
//     }
// }
