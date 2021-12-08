use bevy::prelude::*;

pub mod toggle_switch;

pub struct UiPlugin;

impl Plugin for UiPlugin {
    fn build(&self, app: &mut App) {
        app.init_resource::<Materials>()
            .add_plugin(toggle_switch::ToggleSwitchPlugin)
            .add_startup_system(draw.system())
            .add_system(track_traces_toggle.system())
            .add_system(track_sun_presence_toggle.system());
    }
}

struct Materials {
    bg: Handle<ColorMaterial>,
    toggle_switch: toggle_switch::Materials,
}

impl FromWorld for Materials {
    fn from_world(world: &mut World) -> Self {
        let mut materials = world.get_resource_mut::<Assets<ColorMaterial>>().unwrap();
        Materials {
            bg: materials.add(Color::rgba(0.1, 0.1, 0.1, 0.5).into()),
            toggle_switch: toggle_switch::Materials::from_world(world),
        }
    }
}

#[derive(Component, Clone)]
struct AddSunToggle;

#[derive(Component, Clone)]
struct ShowTracesToggle;

fn draw(mut commands: Commands, ui_materials: Res<Materials>) {
    commands
        .spawn_bundle(NodeBundle {
            style: Style {
                size: Size::new(Val::Px(200.0), Val::Percent(100.0)),
                flex_direction: FlexDirection::ColumnReverse,
                ..Default::default()
            },
            material: ui_materials.bg.clone(),
            ..Default::default()
        })
        .with_children(toggle_switch::draw(
            AddSunToggle,
            toggle_switch::ToggleState(true),
            &ui_materials.toggle_switch,
        ))
        .with_children(toggle_switch::draw(
            ShowTracesToggle,
            toggle_switch::ToggleState(false),
            &ui_materials.toggle_switch,
        ));
}

fn track_traces_toggle(
    query: Query<
        &toggle_switch::ToggleState,
        (With<ShowTracesToggle>, Changed<toggle_switch::ToggleState>),
    >,
) {
    for toggle_state in query.iter() {
        info!("Show traces: {:?}", toggle_state);
    }
}

fn track_sun_presence_toggle(
    query: Query<
        &toggle_switch::ToggleState,
        (With<AddSunToggle>, Changed<toggle_switch::ToggleState>),
    >,
) {
    for toggle_state in query.iter() {
        info!("Add the Sun: {:?}", toggle_state);
    }
}
