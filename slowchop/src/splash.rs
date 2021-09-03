use bevy::ecs::component::Component;
use bevy::ecs::system::QuerySingleError;
use bevy::prelude::*;
use std::fmt::Debug;
use std::hash::Hash;
use std::marker::PhantomData;

pub struct SplashScreen;

#[derive(Debug)]
pub enum SplashScreenConfig {
    Stopped,
    Active {
        timer: Timer,
        image: String,
        entity: Option<Entity>,
    },
}

impl SplashScreenConfig {
    pub fn start(seconds: f32, image: String) -> Self {
        Self::Active {
            timer: Timer::from_seconds(seconds, false),
            image,
            entity: None,
        }
    }
}

impl Plugin for SplashScreen {
    fn build(&self, app: &mut AppBuilder) {
        app.add_startup_system(setup.system());
        app.add_system(run.system());
    }
}

fn setup(mut commands: Commands) {
    commands.spawn_bundle(OrthographicCameraBundle::new_2d());
}

fn run(
    mut commands: Commands,
    time: Res<Time>,
    asset_server: Res<AssetServer>,
    mut materials: ResMut<Assets<ColorMaterial>>,
    mut config: ResMut<SplashScreenConfig>,
    mut keys: ResMut<Input<KeyCode>>,
    mut mouse_button: ResMut<Input<MouseButton>>,
) {
    match &mut *config {
        SplashScreenConfig::Stopped => {
            return;
        }
        SplashScreenConfig::Active {
            timer,
            image,
            entity,
        } => {
            let mut done = false;
            if timer.tick(time.delta()).just_finished() {
                debug!("Splash Finished");
                done = true;
            }

            if entity.is_none() {
                debug!("Loading texture");
                let texture = asset_server.load(image.as_str());
                let material = materials.add(texture.into());
                let ent = commands
                    .spawn_bundle(SpriteBundle {
                        material: material.clone(),
                        ..Default::default()
                    })
                    .id();
                *entity = Some(ent);
            };

            for _ in keys.get_just_pressed() {
                done = true;
                // TODO: keys.reset(key)
            }

            if mouse_button.just_pressed(MouseButton::Left) {
                done = true;
                mouse_button.reset(MouseButton::Left);
            }

            if done {
                if let Some(e) = entity {
                    commands.entity(*e).despawn_recursive();
                    *config = SplashScreenConfig::Stopped;
                }
            }
        }
    }
}
