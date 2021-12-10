use std::ops::{AddAssign, DivAssign, Mul, MulAssign};

use bevy::{core::FixedTimestep, input::mouse::MouseWheel, prelude::*};
use bevy_prototype_lyon::prelude::*;
use ui::{toggle_switch::ToggleState, AddSunToggle, ShowTracesToggle};

pub mod ui;

const CALCULATE_TIME_STEP: f32 = 0.005;
const DRAW_TIME_STEP: f32 = CALCULATE_TIME_STEP * 24.0;

fn main() {
    App::new()
        .add_plugins(DefaultPlugins)
        .add_plugin(ShapePlugin)
        .add_plugin(ui::UiPlugin)
        .init_resource::<ViewScale>()
        .add_startup_system(setup.system())
        .add_system_to_stage(CoreStage::PreUpdate, set_init_sun_velocity.system())
        .add_system(zoom_view.system().label("zoom view"))
        .add_system(scale_object_sizes.system().after("zoom view"))
        .add_system(update_scale_line.system().after("zoom view"))
        .add_system(
            calculate_new_state
                .system()
                .with_run_criteria(FixedTimestep::step(CALCULATE_TIME_STEP as f64)),
        )
        .add_system_set(
            SystemSet::new()
                .with_run_criteria(FixedTimestep::step(DRAW_TIME_STEP as f64))
                // .with_system(list_objects.system().label("list"))
                .with_system(
                    add_trace_point.system().label("add_trace_point"), // .after("list"),
                )
                .with_system(update_trace_point.system().after("add_trace_point")),
        )
        .add_system(add_remove_sun.system())
        .add_system(add_remove_traces.system())
        .run();
}

const MAX_SCALE_LINE_LENGTH: f32 = 200.0;

#[derive(Component, Debug)]
struct ScaleRuler {
    distance: f32,
    unit: String,
    length: f32,
}

#[derive(Clone, Component, Debug)]
struct Position(Vec2);

#[derive(Clone, Component)]
struct Velocity(Vec2);

#[derive(Component)]
struct Star;

#[derive(Component)]
struct Planet;

#[derive(Component)]
struct SetInitVelocity;

#[derive(Clone, Component)]
struct Mass(f32);

#[derive(Component)]
struct Diameter(f32);

#[derive(Component, Debug)]
struct TracePoint {
    position: Vec3,
    drawn: bool,
}

#[derive(Component)]
struct HistoryTracePoint;

#[derive(Component)]
struct ViewScale(f32);

impl Default for ViewScale {
    fn default() -> Self {
        Self(INIT_SCALE)
    }
}

impl TracePoint {
    fn new(position: Position) -> Self {
        Self {
            position: position.0.extend(100.0),
            drawn: false,
        }
    }
}

const G: f32 = 6.67e-11;
const TIME_INTERVAL: f32 = 3600.0;
const ZERO_ANGLE: Vec2 = Vec2::X;
const INIT_SCALE: f32 = 500.0 / 260e9;
const SCALE_CHANGE_BY: f32 = 1.3;

const MIN_STAR_SIZE: f32 = 4.0;
// const MIN_PLANET_SIZE: f32 = 2.0;

#[derive(Clone, Component, Debug)]
struct Name(String);

fn update_trace_point(mut query: Query<(&mut TracePoint, &Position)>) {
    for (mut trace_point, position) in query.iter_mut() {
        trace_point.position = position.0.extend(trace_point.position.z);
        trace_point.drawn = false;
    }
}

#[allow(unused)]
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

fn add_remove_traces(
    mut commands: Commands,
    toggle_query: Query<&ToggleState, (With<ShowTracesToggle>, Changed<ToggleState>)>,
    query: Query<(Entity, &Position, Option<&TracePoint>), With<Mass>>,
    history_points: Query<Entity, With<HistoryTracePoint>>,
) {
    if toggle_query.is_empty() {
        return;
    }

    let turn_on = toggle_query.single().0;

    if !turn_on {
        history_points.for_each(|point_entity| commands.entity(point_entity).despawn());
    }

    for (entity, position, trace_point) in query.iter() {
        let mut entity = commands.entity(entity);

        match (trace_point, turn_on) {
            (Some(_), false) => {
                entity.remove::<TracePoint>();
            }
            (None, true) => {
                entity.insert(TracePoint::new(position.clone()));
            }
            _ => {}
        };
    }
}

fn add_remove_sun(
    mut commands: Commands,
    view_scale: Res<ViewScale>,
    sun_query: Query<Entity, With<Star>>,
    toggle_query: Query<&ToggleState, (With<AddSunToggle>, Changed<ToggleState>)>,
) {
    if toggle_query.is_empty() {
        return;
    }

    let toggle = toggle_query.single();
    let is_sun_present = !sun_query.is_empty();

    if is_sun_present && !toggle.0 {
        let sun = sun_query.single();
        commands.entity(sun).despawn();
    }
    if !is_sun_present && toggle.0 {
        add_sun(commands, &view_scale);
    }
}

fn zoom_view(mut scroll_event: EventReader<MouseWheel>, mut view_scale: ResMut<ViewScale>) {
    for event in scroll_event.iter() {
        let change_by = SCALE_CHANGE_BY * event.y.abs();
        if event.y.is_sign_negative() {
            view_scale.0.mul_assign(change_by);
        } else if event.y.is_sign_positive() {
            view_scale.0.div_assign(change_by);
        }
    }
}

fn scale_object_sizes(
    view_scale: Res<ViewScale>,
    mut query: Query<(&Diameter, &mut Transform, &Name)>,
) {
    if !view_scale.is_changed() {
        return;
    }

    for (diameter, mut transform, name) in query.iter_mut() {
        let calculated = diameter.0 * view_scale.0;

        if calculated > MIN_STAR_SIZE {
            let seen = calculated / MIN_STAR_SIZE;
            transform.scale = Vec3::new(seen, seen, seen)
        }
        println!("{}: scale {}", name.0, transform.scale);
    }
}

fn update_scale_line(view_scale: Res<ViewScale>, mut query: Query<&mut ScaleRuler>) {
    if !view_scale.is_changed() {
        return;
    }

    for mut scale_ruler in query.iter_mut() {
        let meters_per_ruler = MAX_SCALE_LINE_LENGTH / view_scale.0;
        let log10_of_meter = meters_per_ruler.log10().floor();

        let power_of_10 = 10.0_f32.powf(log10_of_meter);
        let distance_meters = (meters_per_ruler / power_of_10).floor() * power_of_10;

        let (unit, distance) = if log10_of_meter < 3.0 {
            ("m", distance_meters)
        } else if log10_of_meter < 6.0 {
            ("km", distance_meters / 1.0e3)
        } else if log10_of_meter < 9.0 {
            ("tnd. km", distance_meters / 1.0e6)
        } else if log10_of_meter < 12.0 {
            ("mln. km", distance_meters / 1.0e9)
        } else {
            ("bln. km", distance_meters / 1.0e12)
        };
        scale_ruler.distance = distance;
        scale_ruler.unit = unit.to_string();
        scale_ruler.length = distance_meters * MAX_SCALE_LINE_LENGTH / meters_per_ruler;
        println!("Scale line  : {:?}", scale_ruler);
    }
}

fn set_init_sun_velocity(
    mut commands: Commands,
    mut query: QuerySet<(
        QueryState<(Entity, &mut Velocity, &Mass), With<SetInitVelocity>>,
        QueryState<(&Mass, &Velocity)>,
    )>,
) {
    if query.q0().iter_mut().count() == 0 {
        return;
    }

    let mut mass_velocity: Vec2 = Vec2::new(0.0, 0.0);

    for (mass, velocity) in query.q1().iter() {
        mass_velocity = mass_velocity + mass.0 * velocity.0;
    }

    let mut query0 = query.q0();
    let (entity, mut velocity, mass) = query0.single_mut();

    let init_velocity = mass_velocity / -mass.0;
    println!("Set init velocity to: {}", init_velocity);

    velocity.0 = init_velocity;
    commands.entity(entity).remove::<SetInitVelocity>();
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
    view_scale: Res<ViewScale>,
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
        transform.translation = (position.0.mul(view_scale.0), 0.0).into();
        // println!("{} ({:?}) => [{:?}]", name.0, position.0, velocity.0);
    }
    // println!("------");
}

fn setup(mut commands: Commands, view_scale: Res<ViewScale>) {
    commands.spawn_bundle(OrthographicCameraBundle::new_2d());
    commands.spawn_bundle(UiCameraBundle::default());

    commands.spawn().insert(ScaleRuler {
        distance: MAX_SCALE_LINE_LENGTH / view_scale.0,
        unit: "m".to_string(),
        length: MAX_SCALE_LINE_LENGTH,
    });

    commands = add_sun(commands, &view_scale);

    commands = add_planet(
        commands,
        Name("Mercury".to_string()),
        Position(Vec2::new(69.817445e9, 0.0)),
        Velocity(Vec2::new(0.0, 38.7e3)),
        Mass(3.285e23),
        view_scale.0,
    );

    commands = add_planet(
        commands,
        Name("Venus".to_string()),
        Position(Vec2::new(-108e9, 0.0)),
        Velocity(Vec2::new(0.0, -35.0e3)),
        Mass(4.867e24),
        view_scale.0,
    );

    commands = add_planet(
        commands,
        Name("Earth".to_string()),
        Position(Vec2::new(0.0, 152.098232e9)),
        Velocity(Vec2::new(-29.4e3, 0.0)),
        Mass(5.9722e24),
        view_scale.0,
    );

    commands = add_planet(
        commands,
        Name("Mars".to_string()),
        Position(Vec2::new(0.0, -249.232e9)),
        Velocity(Vec2::new(22.0e3, 0.0)),
        Mass(6.4171e23),
        view_scale.0,
    );

    commands = add_planet(
        commands,
        Name("Jupiter".to_string()),
        Position(Vec2::new(816.5208e9, 0.0)),
        Velocity(Vec2::new(0.0, 12.0e3)),
        Mass(1.8986e27),
        view_scale.0,
    );

    commands = add_planet(
        commands,
        Name("Saturn".to_string()),
        Position(Vec2::new(0.0, 1513.325783e9)),
        Velocity(Vec2::new(-9.0e3, 0.0)),
        Mass(5.6846e26),
        view_scale.0,
    );

    commands = add_planet(
        commands,
        Name("Uranus".to_string()),
        Position(Vec2::new(-3004.419704e9, 0.0)),
        Velocity(Vec2::new(0.0, -6.0e3)),
        Mass(8.6813e25),
        view_scale.0,
    );

    add_planet(
        commands,
        Name("Neptune".to_string()),
        Position(Vec2::new(0.0, -4553.946490e9)),
        Velocity(Vec2::new(5.4e3, 0.0)),
        Mass(8.6813e25),
        view_scale.0,
    );
}

fn add_trace_point(
    mut commands: Commands,
    view_scale: Res<ViewScale>,
    mut query: Query<&mut TracePoint>,
) {
    let trace_point_shape = shapes::Circle {
        radius: 1.0,
        center: Vec2::new(0.0, 0.0),
    };
    for mut trace in query.iter_mut() {
        if trace.drawn {
            continue;
        }

        let scaled = trace
            .position
            .truncate()
            .mul(view_scale.0)
            .extend(trace.position.z);
        commands.spawn_bundle(GeometryBuilder::build_as(
            &trace_point_shape,
            // ShapeColors::new(Color::DARK_GREEN),
            DrawMode::Fill(FillMode::color(Color::RED)),
            Transform::from_xyz(scaled.x, scaled.y, 0.0),
        )).insert(HistoryTracePoint);

        trace.drawn = true;
    }
}

fn add_planet<'w, 's>(
    mut commands: Commands<'w, 's>,
    name: Name,
    position: Position,
    velocity: Velocity,
    mass: Mass,
    view_scale: f32,
) -> Commands<'w, 's> {
    let shape = shapes::Circle {
        radius: 2.0,
        center: Vec2::new(0.0, 0.0),
    };
    let scaled_position = position.0.mul(view_scale);

    commands
        .spawn_bundle(GeometryBuilder::build_as(
            &shape,
            // ShapeColors::new(Color::BLACK),
            DrawMode::Fill(FillMode::color(Color::BLACK)),
            Transform::from_xyz(scaled_position.x, scaled_position.y, 0.0),
        ))
        .insert(Planet)
        .insert(name)
        .insert(position.clone())
        .insert(velocity)
        .insert(mass);

    commands
}

fn add_sun<'w, 's>(mut commands: Commands<'w, 's>, view_scale: &ViewScale) -> Commands<'w, 's> {
    let sun_position = Position(Vec2::new(0.0, 0.0));
    let sun_diameter = 1.39268e9;

    let sun_circle = shapes::Circle {
        radius: f32::max(MIN_STAR_SIZE, sun_diameter * view_scale.0),
        center: Vec2::new(0.0, 0.0),
    };

    commands
        .spawn_bundle(GeometryBuilder::build_as(
            &sun_circle,
            // ShapeColors::new(Color::YELLOW),
            DrawMode::Fill(FillMode::color(Color::YELLOW)),
            Transform::default(),
        ))
        .insert(Star)
        .insert(SetInitVelocity)
        .insert(Name("Sun".to_string()))
        .insert(sun_position.clone())
        .insert(Velocity(Vec2::new(0.0, 0.0)))
        .insert(Mass(1.989e30))
        .insert(Diameter(sun_diameter));

    commands
}
