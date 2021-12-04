use bevy::prelude::*;

pub mod toggle_switch;

pub struct UiPlugin;

impl Plugin for UiPlugin {
    fn build(&self, app: &mut App) {
        app.init_resource::<Materials>()
            .add_plugin(toggle_switch::ToggleSwitchPlugin)
            .add_startup_system(draw.system());
    }
}


struct Materials {
    bg: Handle<ColorMaterial>,
}

impl FromWorld for Materials {
    fn from_world(world: &mut World) -> Self {
        let mut materials = world.get_resource_mut::<Assets<ColorMaterial>>().unwrap();
        Materials {
            bg: materials.add(Color::rgba(0.1, 0.1, 0.1, 0.5).into()),
        }
    }
}

fn draw(mut commands: Commands, toggle_materials: Res<toggle_switch::Materials>, ui_materials: Res<Materials>) {
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
        .with_children(toggle_switch::draw(&toggle_materials))
        .with_children(toggle_switch::draw(&toggle_materials));
}
