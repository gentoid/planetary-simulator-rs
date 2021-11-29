use std::ops::{AddAssign, Mul};

use bevy::{core::FixedTimestep, prelude::*};
use bevy_prototype_lyon::prelude::*;

const CALCULATE_TIME_STEP: f32 = 0.05;
const DRAW_TIME_STEP: f32 = CALCULATE_TIME_STEP * 24.0;

fn main() {
    App::build()
        .add_plugins(DefaultPlugins)
        .add_plugin(ShapePlugin)
        .add_startup_system(setup.system())
        .add_system(
            calculate_new_state
                .system()
                .with_run_criteria(FixedTimestep::step(CALCULATE_TIME_STEP as f64)),
        )
        .add_system_set(
            SystemSet::new()
                .with_run_criteria(FixedTimestep::step(DRAW_TIME_STEP as f64))
                .with_system(list_objects.system().label("list"))
                .with_system(add_trace_point.system().label("add_trace_point").after("list"))
                .with_system(update_trace_point.system().after("add_trace_point")),
        )
        .run();
}

#[derive(Clone)]
struct Position(Vec2);

#[derive(Clone)]
struct Velocity(Vec2);

struct Star;

struct Planet;

#[derive(Clone)]
struct Mass(f32);

struct TracePoint {
    position: Position,
    drawn: bool,
}

impl TracePoint {
    fn new(position: Position) -> Self {
        Self { position, drawn: false }
    }
}

const G: f32 = 6.67e-11;
const TIME_INTERVAL: f32 = 3600.0;
const ZERO_ANGLE: Vec2 = Vec2::X;
const SCALE: f32 = 500.0 / 260e9;

#[derive(Clone)]
struct Name(String);

fn update_trace_point(mut query: Query<(&mut TracePoint, &Position)>) {
    for (mut trace_point, position) in query.iter_mut() {
        trace_point.position = position.clone();
        trace_point.drawn = false;
    }
}

fn list_objects(query: Query<(&Name, &Position, &Velocity), With<Mass>>) {
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
    let mut prev_state = vec![];

    for (entity, name, position, _, mass, _) in query.iter_mut() {
        prev_state.push((entity.id(), name.clone(), position.clone(), mass.clone()));
    }

    for (entity, _, mut position, mut velocity, _, mut transform) in query.iter_mut() {
        for (other_entity_id, _, other_position, other_mass) in &prev_state {
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

    let sun_position = Position(Vec2::new(0.0, 0.0));

    commands
        .spawn_bundle(GeometryBuilder::build_as(
            &sun_circle,
            ShapeColors::new(Color::YELLOW),
            DrawMode::Fill(FillOptions::default()),
            Transform::default(),
        ))
        .insert(Star)
        .insert(Name("Sun".to_string()))
        .insert(sun_position.clone())
        .insert(Velocity(Vec2::new(0.0, 0.0)))
        .insert(Mass(1.989e30))
        .insert(TracePoint::new(sun_position));

    commands = add_planet(
        commands,
        Name("Mercury".to_string()),
        Position(Vec2::new(69.817445e9, 0.0)),
        Velocity(Vec2::new(0.0, 38.7e3)),
        Mass(3.285e23),
        false,
    );

    commands = add_planet(
        commands,
        Name("Venus".to_string()),
        Position(Vec2::new(-108e9, 0.0)),
        Velocity(Vec2::new(0.0, -35.0e3)),
        Mass(4.867e24),
        false,
    );

    commands = add_planet(
        commands,
        Name("Earth".to_string()),
        Position(Vec2::new(0.0, 152.098232e9)),
        Velocity(Vec2::new(-29.4e3, 0.0)),
        Mass(5.9722e24),
        false,
    );

    add_planet(
        commands,
        Name("Mars".to_string()),
        Position(Vec2::new(0.0, -249.232e9)),
        Velocity(Vec2::new(22.0e3, 0.0)),
        Mass(6.4171e23),
        false,
    );
}

fn add_trace_point(mut commands: Commands, mut query: Query<&mut TracePoint>) {
    let trace_point_shape = shapes::Circle {
        radius: 1.0,
        center: Vec2::new(0.0, 0.0),
    };
    for mut trace in query.iter_mut() {
        if trace.drawn {
            continue;
        }

        let scaled = trace.position.0.mul(SCALE);
        commands.spawn_bundle(GeometryBuilder::build_as(
            &trace_point_shape,
            ShapeColors::new(Color::DARK_GREEN),
            DrawMode::Fill(FillOptions::default()),
            Transform::from_xyz(scaled.x, scaled.y, 0.0),
        ));

        trace.drawn = true;
    }
}

fn add_planet(
    mut commands: Commands,
    name: Name,
    position: Position,
    velocity: Velocity,
    mass: Mass,
    add_trace: bool,
) -> Commands {
    let shape = shapes::Circle {
        radius: 2.0,
        center: Vec2::new(0.0, 0.0),
    };
    let scaled_position = position.0.mul(SCALE);

    let entity = commands
        .spawn_bundle(GeometryBuilder::build_as(
            &shape,
            ShapeColors::new(Color::BLACK),
            DrawMode::Fill(FillOptions::default()),
            Transform::from_xyz(scaled_position.x, scaled_position.y, 0.0),
        ))
        .insert(Planet)
        .insert(name)
        .insert(position.clone())
        .insert(velocity)
        .insert(mass)
        .id();

        if add_trace {
            commands.entity(entity)
            .insert(TracePoint::new(position));
        }

    commands
}
