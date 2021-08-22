use crate::input::exit_on_escape_key;
use crate::AppState;
use bevy::prelude::*;

pub struct MainMenu;

impl Plugin for MainMenu {
    fn build(&self, app: &mut AppBuilder) {
        app
            //
            .add_system_set(SystemSet::on_enter(AppState::MainMenu).with_system(setup.system()))
            .add_system_set(
                SystemSet::on_update(AppState::MainMenu)
                    .with_system(exit_on_escape_key.system())
                    .with_system(hmmm.system()),
            );
    }
}

pub fn setup() {
    debug!("Setting up main menu.")
}

pub fn hmmm() {
    info!("hmmmmmmmm");
}
