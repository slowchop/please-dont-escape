use crate::AppState;
use bevy::prelude::*;

pub struct SplashScreen;

impl Plugin for SplashScreen {
    fn build(&self, app: &mut AppBuilder) {
        app
            //
            .add_system_set(SystemSet::on_enter(AppState::SplashScreen).with_system(setup.system()))
            .add_system_set(
                SystemSet::on_update(AppState::SplashScreen)
                    .with_system(timer.system())
                    .with_system(escape_key.system())
                    .with_system(mouse_click.system())
            )
            .add_system_set(
                SystemSet::on_exit(AppState::SplashScreen).with_system(cleanup.system()),
            );
    }
}

pub struct SplashTimer(Timer);

fn setup(
    mut commands: Commands,
    asset_server: Res<AssetServer>,
    mut materials: ResMut<Assets<ColorMaterial>>,
) {
    info!("setup");
    commands.insert_resource(SplashTimer(Timer::from_seconds(2.0, false)));

    commands.spawn_bundle(OrthographicCameraBundle::new_2d());

    // Test sprite is 887x459. It should/will be 1920x1080.
    let handle = asset_server.load("splash/splash.png");
    commands.spawn_bundle(SpriteBundle {
        material: materials.add(handle.into()),
        ..Default::default()
    });
}

fn set_next_state(state: &mut ResMut<State<AppState>>) {
    state
        .set(AppState::MainMenu)
        .expect("Could not set state to MainMenu.");
}

fn timer(mut state: ResMut<State<AppState>>, time: Res<Time>, mut timer: ResMut<SplashTimer>) {
    if timer.0.tick(time.delta()).just_finished() {
        set_next_state(&mut state);
    }
}

fn escape_key(mut state: ResMut<State<AppState>>, mut keys: ResMut<Input<KeyCode>>) {
    if keys.just_pressed(KeyCode::Escape) {
        // Reset so that the main menu doesn't get an escape key event immediately.
        keys.reset(KeyCode::Escape);
        set_next_state(&mut state);
    }
}

fn mouse_click(mut state: ResMut<State<AppState>>, mut mouse_button: ResMut<Input<MouseButton>>) {
    if mouse_button.just_pressed(MouseButton::Left) {
        mouse_button.reset(MouseButton::Left);
        set_next_state(&mut state);
    }
}

fn cleanup(mut commands: Commands, entities: Query<Entity>) {
    debug!("Cleanup splash.");
    for sprite in entities.iter() {
        commands.entity(sprite).despawn_recursive();
    }
}
