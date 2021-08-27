use bevy::prelude::*;
use rand::{RngCore, thread_rng};
use rand::prelude::IteratorRandom;
use crate::game;
use crate::game::{Alpha, Door};

#[derive(Debug)]
pub struct Smoke;

#[derive(Debug)]
pub struct Smoking(Timer);


pub fn damage_wires(
    mut commands: Commands,
    good_wires: Query<Entity, (With<Wire>, Without<Damaged>, Without<Broken>)>,
) {
    let mut rng = thread_rng();
    // 1000 seems good
    // 100 is good for testing
    if rng.next_u32() % 1000 != 0 {
        return;
    }

    let entities = good_wires.iter().choose_multiple(&mut rng, 1);
    let ent = entities.get(0);
    info!("damaging: {:?}", ent);
    match ent {
        Some(e) => {
            commands
                .entity(*e)
                .insert(Damaged(Timer::from_seconds(2.0, false)))
                .insert(Smoking(Timer::from_seconds(0.5, true)));
        }
        None => {
            info!("No wires left to smoke");
        }
    };
}

pub fn damaged_smoke(
    mut commands: Commands,
    mut materials: ResMut<Assets<ColorMaterial>>,
    asset_server: Res<AssetServer>,
    time: Res<Time>,
    mut damaged: Query<(&Transform, &mut Smoking)>,
) {
    for (transform, mut timer) in damaged.iter_mut() {
        if !timer.0.tick(time.delta()).just_finished() {
            continue;
        }

        let mut color_material: ColorMaterial = asset_server.load("effects/smoke.png").into();
        let material = materials.add(color_material);
        commands
            .spawn_bundle(SpriteBundle {
                material: material.clone(),
                transform: transform.clone(),
                ..Default::default()
            })
            .insert(Smoke)
            .insert(Alpha(1.0));
    }
}

pub fn open_doors_if_any_wires_are_broken(
    mut commands: Commands,
    broken_wires: Query<(Entity), (With<Wire>, With<Broken>)>,
    doors: Query<Entity, With<Door>>,
) {
    let mut found = false;
    for _ in broken_wires.iter() {
        found = true;
        break;
    }
    if !found {
        return;
    }

    for door_ent in doors.iter() {
        game::change_door_state(&mut commands, door_ent, true);
    }
}

pub fn damaged_check_if_broken(
    mut commands: Commands,
    mut materials: ResMut<Assets<ColorMaterial>>,
    asset_server: Res<AssetServer>,
    time: Res<Time>,
    mut damaged: Query<(Entity, &mut Damaged)>,
) {
    for (ent, mut damage) in damaged.iter_mut() {
        if !damage.0.tick(time.delta()).just_finished() {
            continue;
        }

        let mut color_material: ColorMaterial = asset_server.load("cells/wire-broken.png").into();
        let material = materials.add(color_material);
        commands
            .entity(ent)
            .remove::<Damaged>()
            .remove::<Smoking>()
            .insert(Broken)
            .insert(material);
    }
}

pub fn move_smoke(
    mut commands: Commands,
    mut materials: ResMut<Assets<ColorMaterial>>,
    asset_server: Res<AssetServer>,
    mut smokes: Query<
        (
            Entity,
            &mut Transform,
            &mut Handle<ColorMaterial>,
            &mut Alpha,
        ),
        With<Smoke>,
    >,
) {
    for (ent, mut transform, mut material, mut alpha) in smokes.iter_mut() {
        alpha.0 -= 0.01;
        if alpha.0 <= 0.0 {
            commands.entity(ent).despawn();
            return;
        }

        // none of this works :(
        //
        // let mut color_mat = materials.get_mut(&material).unwrap();
        // color_mat.color = Color::rgba(1.0,0.0,1.0, alpha.0);
        // ColorMaterial::modulated_texture()
        // color_material.color = Color::Rgba {
        //     red: 1.0,
        //     green: 0.0,
        //     blue: 0.0,
        //     alpha: alpha.0,
        // };
        // *material = materials.add(color_material);
        // dbg!("yay");
        transform.translation.y += 0.1;
        // let new_alpha = material.color.a() - 0.001;
        // material.color.set_a(new_alpha);
        // dbg!(material.color);
    }
}

#[derive(Debug)]
pub struct Wire;

#[derive(Debug)]
pub struct Damaged(Timer);

#[derive(Debug)]
pub struct Broken;
