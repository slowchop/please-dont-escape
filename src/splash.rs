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
                    .with_system(escape_key.system()),
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
    commands.insert_resource(SplashTimer(Timer::from_seconds(2.0, false)));

    commands.spawn_bundle(OrthographicCameraBundle::new_2d());

    // Test sprite is 887x459. It should/will be 1920x1080.
    let handle = asset_server.load("splash/splash.png");
    commands.spawn_bundle(SpriteBundle {
        material: materials.add(handle.into()),
        ..Default::default()
    });
}

fn timer(time: Res<Time>, mut timer: ResMut<SplashTimer>, mut state: ResMut<State<AppState>>) {
    if timer.0.tick(time.delta()).just_finished() {
        state
            .set(AppState::MainMenu)
            .expect("Could not set state to MainMenu");
    }
}

fn escape_key(keys: Res<Input<KeyCode>>, mut state: ResMut<State<AppState>>) {
    if keys.just_pressed(KeyCode::Escape) {
        state
            .set(AppState::MainMenu)
            .expect("Could not set state to MainMenu");
    }
}

fn cleanup(mut commands: Commands, sprites: Query<Entity>) {
    debug!("Cleanup splash.");
    for sprite in sprites.iter() {
        commands.entity(sprite).despawn();
    }
}
