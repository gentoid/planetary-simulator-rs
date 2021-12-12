use std::ops::{Add, AddAssign, Div, DivAssign, Mul, MulAssign, Sub};

use bevy::{core::FixedTimestep, input::mouse::MouseWheel, prelude::*};
use bevy_prototype_lyon::prelude::*;
use ui::{toggle_switch::ToggleState, AddSunToggle, ShowTracesToggle};

pub mod ui;

const CALCULATE_TIME_STEP: f32 = 0.001;
const DRAW_TIME_STEP: f32 = CALCULATE_TIME_STEP * 240.0;

fn main() {
    App::new()
        .add_plugins(DefaultPlugins)
        .add_plugin(ShapePlugin)
        .add_plugin(ui::UiPlugin)
        .init_resource::<ViewScale>()
        .add_event::<NewTracePointDrawn>()
        .add_event::<NeedToAdjustSunVelocity>()
        .add_startup_system(setup.system())
        .add_system_to_stage(CoreStage::PreUpdate, adjust_sun_velocity.system())
        .add_system(zoom_view.system().label("zoom view"))
        .add_system(scale_object_sizes.system().after("zoom view"))
        .add_system(update_scale_line.system().after("zoom view"))
        .add_system(zoom_trace_lines.system().after("zoom view"))
        .add_system(
            calculate_new_state
                .system()
                .with_run_criteria(FixedTimestep::step(CALCULATE_TIME_STEP as f64)),
        )
        .add_system_set(
            SystemSet::new()
                .with_run_criteria(FixedTimestep::step(DRAW_TIME_STEP as f64))
                .with_system(set_baricenters.system())
                // .with_system(list_objects.system().label("list"))
                .with_system(draw_trace_point.system()),
        )
        .add_system(add_remove_sun.system())
        .add_system(add_remove_traces.system())
        .add_system(on_new_trace_point.system())
        .run();
}

const MAX_SCALE_LINE_LENGTH: f32 = 200.0;

#[derive(Component, Debug)]
struct ScaleRuler {
    distance: f32,
    unit: String,
    length: f32,
}

#[derive(Clone, Component, Debug, Default)]
struct Position(Vec3);

impl Sub for &Position {
    type Output = Position;

    fn sub(self, other: Self) -> Self::Output {
        Position(self.0 - other.0)
    }
}

impl Sub<Position> for &Position {
    type Output = Position;

    fn sub(self, other: Position) -> Self::Output {
        Position(self.0 - other.0)
    }
}

impl Sub<&Self> for Position {
    type Output = Self;

    fn sub(self, other: &Self) -> Self::Output {
        Self(self.0 - other.0)
    }
}

impl Sub<Self> for Position {
    type Output = Self;

    fn sub(self, other: Self) -> Self::Output {
        Self(self.0 - other.0)
    }
}

impl Add for &Position {
    type Output = Position;

    fn add(self, other: Self) -> Self::Output {
        Position(self.0 + other.0)
    }
}

impl Add<&Self> for Position {
    type Output = Self;

    fn add(self, other: &Self) -> Self::Output {
        Self(self.0 + other.0)
    }
}

impl Mul<f32> for Position {
    type Output = Self;

    fn mul(self, other: f32) -> Self::Output {
        Self(self.0 * other)
    }
}

impl Div<f32> for Position {
    type Output = Self;

    fn div(self, other: f32) -> Self::Output {
        Self(self.0 / other)
    }
}

impl Mul<&Res<'_, ViewScale>> for Position {
    type Output = Self;

    fn mul(self, other: &Res<'_, ViewScale>) -> Self::Output {
        Self(self.0 * other.0)
    }
}

impl From<Position> for Vec3 {
    fn from(position: Position) -> Self {
        position.0
    }
}

#[derive(Clone, Component)]
struct Velocity(Vec3);

impl Sub for &Velocity {
    type Output = Velocity;

    fn sub(self, other: Self) -> Self::Output {
        Velocity(self.0 - other.0)
    }
}

#[derive(Component)]
struct Star;

#[derive(Component)]
struct Planet;

enum SystemChanged {
    BodyAdded,
    BodyRemoved,
}

struct NeedToAdjustSunVelocity {
    change: SystemChanged,
    mass: Mass,
    velocity: Velocity,
}

#[derive(Clone, Component)]
struct Mass(f32);

impl Add for &Mass {
    type Output = Mass;

    fn add(self, other: Self) -> Self::Output {
        Mass(self.0 + other.0)
    }
}

impl Add for Mass {
    type Output = Self;

    fn add(self, other: Self) -> Self::Output {
        Self(self.0 + other.0)
    }
}

impl Div for &Mass {
    type Output = f32;

    fn div(self, other: Self) -> Self::Output {
        self.0 / other.0
    }
}

#[derive(Component)]
struct Diameter(f32);

#[derive(Component, Debug)]
struct TraceLine {
    pub points: Vec<Entity>,
    draw: bool,
}

impl Default for TraceLine {
    fn default() -> Self {
        Self {
            points: Vec::with_capacity(100),
            draw: false,
        }
    }
}

impl TraceLine {
    fn add(&mut self, point_entity: Entity) -> Vec<Entity> {
        let mut removed = vec![];

        self.points.push(point_entity);

        while self.points.len() >= 100 {
            removed.push(self.points.remove(0));
        }

        removed
    }
}

#[derive(Component)]
struct TracePoint;

#[derive(Debug)]
struct NewTracePointDrawn {
    object_entity: Entity,
    point_entity: Entity,
}

#[derive(Component)]
struct ViewScale(f32);

impl Default for ViewScale {
    fn default() -> Self {
        Self(INIT_SCALE)
    }
}

const G: f32 = 6.67e-11;
const SUN_SGP: f32 = 1.32712440019e20; // Standard gravitational parameter
const TIME_INTERVAL: f32 = 3600.0;
const ZERO_ANGLE: Vec3 = Vec3::X;
const INIT_SCALE: f32 = 500.0 / 260e9;
const SCALE_CHANGE_BY: f32 = 1.3;

const MIN_STAR_SIZE: f32 = 4.0;
// const MIN_PLANET_SIZE: f32 = 2.0;

#[derive(Clone, Component, Debug)]
struct Name(String);

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
    mut query: Query<&mut TraceLine>,
) {
    if toggle_query.is_empty() {
        return;
    }

    let turn_on = toggle_query.single().0;

    for mut trace in query.iter_mut() {
        trace.draw = turn_on;

        if turn_on {
            continue;
        }

        while trace.points.len() > 0 {
            let point_entity = trace.points.remove(0);
            commands.entity(point_entity).despawn();
        }
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

fn zoom_trace_lines(
    view_scale: Res<ViewScale>,
    mut query: Query<(&mut Transform, &Position), With<TracePoint>>,
) {
    if !view_scale.is_changed() {
        return;
    }

    for (mut transform, position) in query.iter_mut() {
        transform.translation = (
            position.0.mul(view_scale.0).truncate(),
            transform.translation.z,
        )
            .into();
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

fn adjust_sun_velocity(
    mut adjust_sun_velocity_event: EventReader<NeedToAdjustSunVelocity>,
    mut query: Query<(&mut Velocity, &Mass), With<Star>>,
) {
    if query.is_empty() {
        return;
    }

    let (mut velocity, mass) = query.single_mut();

    for event in adjust_sun_velocity_event.iter() {
        let velocity_diff = event.velocity.0 * event.mass.0 / mass.0;

        let sign = match event.change {
            SystemChanged::BodyAdded => -1.0,
            SystemChanged::BodyRemoved => 1.0,
        };

        info!("Sun's velocity adjusted: {:?}", sign * velocity_diff);

        velocity.0.add_assign(sign * velocity_diff);
    }
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
            let angle = ZERO_ANGLE
                .truncate()
                .angle_between((other_position.0 - position.0).truncate());
            let velocity_diff_length = acceleration * TIME_INTERVAL;
            let velocity_diff = Vec3::new(
                velocity_diff_length * angle.cos(),
                velocity_diff_length * angle.sin(),
                0.0,
            );
            velocity.0.add_assign(velocity_diff);

            // println!(
            //     "{} to {} acc: {:e}, angle: {:e}, velocity diff: ({})",
            //     name.0, other_name.0, acceleration, angle, velocity_diff,
            // );
        }
        position.0.add_assign(velocity.0 * TIME_INTERVAL);
        transform.translation = (
            position.0.mul(view_scale.0).truncate(),
            transform.translation.z,
        )
            .into();
        // println!("{} ({:?}) => [{:?}]", name.0, position.0, velocity.0);
    }
    // println!("------");
}

fn setup(
    mut commands: Commands,
    view_scale: Res<ViewScale>,
    mut adjust_sun_velocity_event: EventWriter<NeedToAdjustSunVelocity>,
) {
    commands.spawn_bundle(OrthographicCameraBundle::new_2d());
    commands.spawn_bundle(UiCameraBundle::default());

    commands.spawn().insert(ScaleRuler {
        distance: MAX_SCALE_LINE_LENGTH / view_scale.0,
        unit: "m".to_string(),
        length: MAX_SCALE_LINE_LENGTH,
    });

    commands = add_sun(commands, &view_scale);

    let planets_data = [
        (
            Name("Mercury".to_string()),
            Position(Vec3::new(69.817445e9, 0.0, 0.0)),
            Velocity(Vec3::new(0.0, 38.7e3, 0.0)),
            Mass(3.285e23),
        ),
        (
            Name("Venus".to_string()),
            Position(Vec3::new(-108e9, 0.0, 0.0)),
            Velocity(Vec3::new(0.0, -35.0e3, 0.0)),
            Mass(4.867e24),
        ),
        (
            Name("Earth".to_string()),
            Position(Vec3::new(0.0, 152.098232e9, 0.0)),
            Velocity(Vec3::new(-29.4e3, 0.0, 0.0)),
            Mass(5.9722e24),
        ),
        (
            Name("Mars".to_string()),
            Position(Vec3::new(0.0, -249.232e9, 0.0)),
            Velocity(Vec3::new(22.0e3, 0.0, 0.0)),
            Mass(6.4171e23),
        ),
        (
            Name("Jupiter".to_string()),
            Position(Vec3::new(816.5208e9, 0.0, 0.0)),
            Velocity(Vec3::new(0.0, 12.0e3, 0.0)),
            Mass(1.8986e27),
        ),
        (
            Name("Saturn".to_string()),
            Position(Vec3::new(0.0, 1513.325783e9, 0.0)),
            Velocity(Vec3::new(-9.0e3, 0.0, 0.0)),
            Mass(5.6846e26),
        ),
        (
            Name("Uranus".to_string()),
            Position(Vec3::new(-3004.419704e9, 0.0, 0.0)),
            Velocity(Vec3::new(0.0, -6.0e3, 0.0)),
            Mass(8.6813e25),
        ),
        (
            Name("Neptune".to_string()),
            Position(Vec3::new(0.0, -4553.946490e9, 0.0)),
            Velocity(Vec3::new(5.4e3, 0.0, 0.0)),
            Mass(8.6813e25),
        ),
    ];

    let shape = shapes::Circle {
        radius: 2.0,
        center: Vec2::new(0.0, 0.0),
    };

    for (name, position, velocity, mass) in planets_data.into_iter() {
        let scaled_position = position.0 * view_scale.0;

        commands
            .spawn_bundle(GeometryBuilder::build_as(
                &shape,
                DrawMode::Fill(FillMode::color(Color::BLACK)),
                Transform::from_xyz(scaled_position.x, scaled_position.y, 50.0),
            ))
            .insert(TraceLine::default())
            .insert(Planet)
            .insert(name)
            .insert(position)
            .insert(velocity.clone())
            .insert(mass.clone());

        adjust_sun_velocity_event.send(NeedToAdjustSunVelocity {
            change: SystemChanged::BodyAdded,
            velocity,
            mass,
        });
    }
}

fn draw_trace_point(
    mut commands: Commands,
    view_scale: Res<ViewScale>,
    mut new_trace_point_event: EventWriter<NewTracePointDrawn>,
    mut query: Query<(Entity, &Position, &TraceLine)>,
) {
    let trace_point_shape = shapes::Circle {
        radius: 1.0,
        center: Vec2::new(0.0, 0.0),
    };
    for (object_entity, position, trace) in query.iter_mut() {
        if !trace.draw {
            continue;
        }

        let scaled = position.0.mul(view_scale.0);

        let point_entity = commands
            .spawn_bundle(GeometryBuilder::build_as(
                &trace_point_shape,
                // ShapeColors::new(Color::DARK_GREEN),
                DrawMode::Fill(FillMode::color(Color::RED)),
                Transform::from_xyz(scaled.x, scaled.y, 0.0),
            ))
            .insert(TracePoint)
            .insert(position.clone())
            .id();

        new_trace_point_event.send(NewTracePointDrawn {
            object_entity,
            point_entity,
        });
    }
}

fn on_new_trace_point(
    mut commands: Commands,
    mut new_trace_point_event: EventReader<NewTracePointDrawn>,
    mut query: Query<(Entity, &mut TraceLine)>,
) {
    for event in new_trace_point_event.iter() {
        for (entity, mut trace) in query.iter_mut() {
            if entity == event.object_entity {
                for point_entity in trace.add(event.point_entity).into_iter() {
                    commands.entity(point_entity).despawn();
                }
            }
        }
    }
}

fn add_sun<'w, 's>(mut commands: Commands<'w, 's>, view_scale: &ViewScale) -> Commands<'w, 's> {
    let sun_position = Position(Vec3::new(0.0, 0.0, 0.0));
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
        .insert(TraceLine::default())
        .insert(Star)
        .insert(Name("Sun".to_string()))
        .insert(sun_position.clone())
        .insert(Velocity(Vec3::new(0.0, 0.0, 0.0)))
        .insert(Mass(1.989e30))
        .insert(Diameter(sun_diameter));

    commands
}

#[derive(Component)]
struct Orbit;

fn set_baricenters(
    mut commands: Commands,
    view_scale: Res<ViewScale>,
    mut planets_query: Query<(&Name, &Position, &Velocity, &Mass), (With<Planet>, Without<Star>)>,
    sun_query: Query<(&Position, &Velocity, &Mass), With<Star>>,
    orbits_query: Query<Entity, With<Orbit>>,
) {
    if sun_query.is_empty() {
        return;
    }

    orbits_query.for_each(|orbit| commands.entity(orbit).despawn());

    let (sun_position, sun_velocity, sun_mass) = sun_query.single();

    for (_, planet_position, planet_velocity, planet_mass) in planets_query.iter_mut() {
        let distance_vector = planet_position - sun_position;

        let baricenter = distance_vector * (planet_mass / &(sun_mass + planet_mass)) + sun_position;

        let position_vector = planet_position - &baricenter;
        let velocity_vector = planet_velocity - sun_velocity;
        let distance = position_vector.0.length();
        let velocity_squared = velocity_vector.0.length_squared();

        let semi_major_axis_length =
            SUN_SGP * distance / (2.0 * SUN_SGP - distance * velocity_squared);

        let h = position_vector.0.cross(velocity_vector.0); // specific angular momentum

        let eccentricity_vector =
            position_vector.0 / distance - (velocity_vector.0.cross(h) / SUN_SGP);

        let semi_minor_axis_length =
            semi_major_axis_length * (1.0 - eccentricity_vector.length_squared()).sqrt();

        let empty_focus = 2.0 * semi_major_axis_length * eccentricity_vector;

        let center = (empty_focus - baricenter.0) / 2.0;

        let ellipsis_angle = ZERO_ANGLE
            .truncate()
            .angle_between(eccentricity_vector.truncate());

        let mut transform = Transform::from_translation(center * view_scale.0);
        transform.rotate(Quat::from_rotation_z(ellipsis_angle));
        commands
            .spawn_bundle(GeometryBuilder::build_as(
                &shapes::Ellipse {
                    center: Vec2::default(),
                    radii: Vec2::new(
                        semi_major_axis_length * view_scale.0,
                        semi_minor_axis_length * view_scale.0,
                    ),
                },
                DrawMode::Stroke(StrokeMode::color(Color::INDIGO)),
                transform,
            ))
            .insert(Orbit);

        // commands.spawn_bundle(GeometryBuilder::build_as(
        //     &shapes::Circle {
        //         radius: 2.0,
        //         center: Vec2::new(0.0, 0.0),
        //     },
        //     DrawMode::Fill(FillMode::color(Color::INDIGO)),
        //     Transform::from_xyz(
        //         empty_focus.x * view_scale.0,
        //         empty_focus.y * view_scale.0,
        //         10.0,
        //     ),
        // ));
    }
}
