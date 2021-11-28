use std::ops::AddAssign;

use bevy::prelude::*;

fn main() {
    App::build()
        .add_plugins(DefaultPlugins)
        .insert_resource(ListTimer(Timer::from_seconds(0.24, true)))
        .insert_resource(StateTimer(Timer::from_seconds(0.01, true)))
        .add_startup_system(add_objects.system())
        .add_system(calculate_new_state.system())
        .add_system(list_objects.system())
        .run();
}

struct ListTimer(Timer);

struct StateTimer(Timer);

#[derive(Clone)]
struct Position(Vec2);

#[derive(Clone)]
struct Velocity(Vec2);

struct Star;

struct NotStar;

#[derive(Clone)]
struct Mass(f32);

const G: f32 = 6.67e-11;
const TIME_INTERVAL: f32 = 3600.0;
const ZERO_ANGLE: Vec2 = Vec2::X;

#[derive(Clone)]
struct Name(String);

fn list_objects(
    time: Res<Time>,
    mut timer: ResMut<ListTimer>,
    query: Query<(&Name, &Position, &Velocity), With<Mass>>,
) {
    if !timer.0.tick(time.delta()).just_finished() {
        return;
    }

    for (name, position, velocity) in query.iter() {
        println!(
            "{} ({}) [{:4.2e}] => ({})",
            name.0,
            position.0,
            position.0.length(),
            velocity.0.length()
        );
    }
    println!("======");
}

fn calculate_new_state(
    time: Res<Time>,
    mut timer: ResMut<StateTimer>,
    mut query: Query<(Entity, &Name, &mut Position, &mut Velocity, &Mass), With<Mass>>,
) {
    if !timer.0.tick(time.delta()).just_finished() {
        return;
    }

    let mut prev_state = vec![];

    for (entity, name, position, _, mass) in query.iter_mut() {
        prev_state.push((entity.id(), name.clone(), position.clone(), mass.clone()));
    }

    for (entity, name, mut position, mut velocity, _) in query.iter_mut() {
        for (other_entity_id, other_name, other_position, other_mass) in &prev_state {
            if &entity.id() == other_entity_id {
                continue;
            }

            // let force = G *  mass.0 * other_mass.0 / square_distance(&position, other_position);
            let acceleration = G * other_mass.0 / position.0.distance_squared(other_position.0);
            let angle = ZERO_ANGLE.angle_between(other_position.0 - position.0);
            let velocity_diff_length = acceleration * TIME_INTERVAL;
            let velocity_diff = Vec2::new(
                velocity_diff_length * angle.cos(),
                velocity_diff_length * angle.sin(),
            );
            velocity.0.add_assign(velocity_diff);

            // println!(
            //     "{} to {} acc: {:e}, angle: {:e}, velocity diff: ({})",
            //     name.0, other_name.0, acceleration, angle, velocity_diff,
            // );
        }
        position.0.add_assign(velocity.0 * TIME_INTERVAL);
        // println!("{} ({:?}) => [{:?}]", name.0, position.0, velocity.0);
    }
    // println!("------");
}

fn add_objects(mut commands: Commands) {
    commands
        .spawn()
        .insert(Star)
        .insert(Name("Sun".to_string()))
        .insert(Position(Vec2::new(0.0, 0.0)))
        .insert(Velocity(Vec2::new(0.0, 0.0)))
        .insert(Mass(1.989e30));
    commands
        .spawn()
        .insert(NotStar)
        .insert(Name("Mercury".to_string()))
        .insert(Position(Vec2::new(69.817445e9, 0.0)))
        .insert(Velocity(Vec2::new(0.0, 38.7e3)))
        .insert(Mass(3.285e23));
    commands
        .spawn()
        .insert(NotStar)
        .insert(Name("Venus".to_string()))
        .insert(Position(Vec2::new(-108e9, 0.0)))
        .insert(Velocity(Vec2::new(0.0, -35.0e3)))
        .insert(Mass(4.867e24));
}
