use bevy::prelude::*;

pub mod toggle_switch;

pub struct UiPlugin;

impl Plugin for UiPlugin {
    fn build(&self, app: &mut App) {
        app.init_resource::<Materials>()
            .init_resource::<UiState>()
            .add_plugin(toggle_switch::ToggleSwitchPlugin)
            .add_startup_system(draw.system());
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

struct UiState {
    add_sun_toggle: toggle_switch::ToggleState,
    show_traces_toggle: toggle_switch::ToggleState,
}

impl Default for UiState {
    fn default() -> Self {
        Self {
            add_sun_toggle: toggle_switch::ToggleState(true),
            show_traces_toggle: toggle_switch::ToggleState(false),
        }
    }
}

#[derive(Component, Clone)]
struct AddSunToggle;

#[derive(Component, Clone)]
struct ShowTracesToggle;

fn draw(mut commands: Commands, ui_materials: Res<Materials>, state: Res<UiState>) {
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
            state.add_sun_toggle,
            &ui_materials.toggle_switch,
        ))
        .with_children(toggle_switch::draw(
            ShowTracesToggle,
            state.show_traces_toggle,
            &ui_materials.toggle_switch,
        ));
}
