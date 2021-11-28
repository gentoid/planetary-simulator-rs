use std::ops::{AddAssign, Mul};

use bevy::prelude::*;
use bevy_prototype_lyon::prelude::*;

fn main() {
    App::build()
        .add_plugins(DefaultPlugins)
        .add_plugin(ShapePlugin)
        .insert_resource(ListTimer(Timer::from_seconds(0.24, true)))
        .insert_resource(StateTimer(Timer::from_seconds(0.01, true)))
        .add_startup_system(setup.system())
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

struct Planet;

#[derive(Clone)]
struct Mass(f32);

const G: f32 = 6.67e-11;
const TIME_INTERVAL: f32 = 3600.0;
const ZERO_ANGLE: Vec2 = Vec2::X;
const SCALE: f32 = 500.0 / 160e9;

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
    mut query: Query<
        (
            Entity,
            &Name,
            &mut Position,
            &mut Velocity,
            &Mass,
            &mut Transform,
        ),
        With<Mass>,
    >,
) {
    if !timer.0.tick(time.delta()).just_finished() {
        return;
    }

    let mut prev_state = vec![];

    for (entity, name, position, _, mass, _) in query.iter_mut() {
        prev_state.push((entity.id(), name.clone(), position.clone(), mass.clone()));
    }

    for (entity, name, mut position, mut velocity, _, mut transform) in query.iter_mut() {
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
        transform.translation = (position.0.mul(SCALE), 0.0).into();
        // println!("{} ({:?}) => [{:?}]", name.0, position.0, velocity.0);
    }
    // println!("------");
}

fn setup(mut commands: Commands) {
    commands.spawn_bundle(OrthographicCameraBundle::new_2d());
    commands.spawn_bundle(UiCameraBundle::default());

    let sun_circle = shapes::Circle {
        radius: 4.0,
        center: Vec2::new(0.0, 0.0),
    };

    commands
        .spawn_bundle(GeometryBuilder::build_as(
            &sun_circle,
            ShapeColors::new(Color::YELLOW),
            DrawMode::Fill(FillOptions::default()),
            Transform::default(),
        ))
        .insert(Star)
        .insert(Name("Sun".to_string()))
        .insert(Position(Vec2::new(0.0, 0.0)))
        .insert(Velocity(Vec2::new(0.0, 0.0)))
        .insert(Mass(1.989e30));

    commands = add_planet(
        commands,
        Name("Mercury".to_string()),
        Position(Vec2::new(69.817445e9, 0.0)),
        Velocity(Vec2::new(0.0, 38.7e3)),
        Mass(3.285e23),
    );

    commands = add_planet(commands,
        Name("Venus".to_string()),
        Position(Vec2::new(-108e9, 0.0)),
        Velocity(Vec2::new(0.0, -35.0e3)),
        Mass(4.867e24),
    );

    add_planet(commands,
        Name("Earth".to_string()),
        Position(Vec2::new(0.0, 152.098232e9)),
        Velocity(Vec2::new(-29.4e3, 0.0)),
        Mass(4.867e24),
    );
}

fn add_planet(
    mut commands: Commands,
    name: Name,
    position: Position,
    velocity: Velocity,
    mass: Mass,
) -> Commands {
    let shape = shapes::Circle {
        radius: 2.0,
        center: Vec2::new(0.0, 0.0),
    };
    let scaled_position = position.0.mul(SCALE);

    commands
        .spawn_bundle(GeometryBuilder::build_as(
            &shape,
            ShapeColors::new(Color::BLACK),
            DrawMode::Fill(FillOptions::default()),
            Transform::from_xyz(scaled_position.x, scaled_position.y, 0.0),
        ))
        .insert(Planet)
        .insert(name)
        .insert(position)
        .insert(velocity)
        .insert(mass);

    commands
}
