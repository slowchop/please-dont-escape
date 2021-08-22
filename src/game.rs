use bevy::core::{FixedTimestep, FixedTimesteps};
use bevy::prelude::*;
use bevy::app::{Events, AppExit};
use crate::input::input_exit_system;

pub struct PleaseDontEscape;

impl Plugin for PleaseDontEscape {
    fn build(&self, app: &mut AppBuilder) {
        app
            //
            // .add_startup_system(setup_scene_system.system())
            // .add_startup_system(spawn_entities_system.system())
            //
            // .add_system(hello_world_system.system())
            .add_system(input_exit_system.system())
            // Fixed timestamp systems!
            // https://github.com/bevyengine/bevy/blob/latest/examples/ecs/fixed_timestep.rs
            .add_stage_after(
                CoreStage::Update,
                FixedUpdateStage,
                SystemStage::parallel()
                    .with_run_criteria(FixedTimestep::step(1f64 / 60f64))
                    .with_system(fixed_system.system()),
            );
    }
}

#[derive(Debug, Hash, PartialEq, Eq, Clone, StageLabel)]
struct FixedUpdateStage;

fn fixed_system() {
    println!("fixed");
}

