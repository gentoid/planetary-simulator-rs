use bevy::prelude::*;

pub struct TogglePlugin;

impl Plugin for TogglePlugin {
    fn build(&self, app: &mut App) {
        app.init_resource::<ToggleMaterials>()
            .add_startup_system(initial_draw.system())
            .add_system(switch_toggle.system());
    }
}

#[derive(Component)]
struct ToggleState(bool);

#[derive(Component)]
struct SliderParent;

#[derive(Component)]
struct SliderBorder;

#[derive(Component)]
struct SliderBody;

struct ToggleMaterials {
    slider_enabled: Handle<ColorMaterial>,
    slider_disabled: Handle<ColorMaterial>,
    border_disabled: Handle<ColorMaterial>,
    border_enabled: Handle<ColorMaterial>,
    bg: Handle<ColorMaterial>,
}

impl FromWorld for ToggleMaterials {
    fn from_world(world: &mut World) -> Self {
        let mut materials = world.get_resource_mut::<Assets<ColorMaterial>>().unwrap();
        ToggleMaterials {
            slider_enabled: materials.add(Color::rgb(0.0, 0.7, 0.0).into()),
            slider_disabled: materials.add(Color::rgb(0.25, 0.25, 0.25).into()),
            border_disabled: materials.add(Color::rgb(0.2, 0.2, 0.2).into()),
            border_enabled: materials.add(Color::DARK_GREEN.into()),
            bg: materials.add(Color::BLACK.into()),
            // pressed: materials.add(Color::rgb(0.35, 0.75, 0.35).into()),
        }
    }
}

fn switch_toggle(
    mouse_click: Res<Input<MouseButton>>,
    materials: Res<ToggleMaterials>,
    mut query: QuerySet<(
        QueryState<&mut ToggleState>,
        QueryState<&mut Style, With<SliderParent>>,
        QueryState<&mut Handle<ColorMaterial>, With<SliderBorder>>,
        QueryState<&mut Handle<ColorMaterial>, With<SliderBody>>,
    )>,
) {
    if mouse_click.just_pressed(MouseButton::Left) {
        let mut query0 = query.q0();
        let mut toggle_state = query0.single_mut();

        let is_enabled = !toggle_state.0;
        toggle_state.0 = is_enabled;

        let new_border_style = if is_enabled {
            JustifyContent::FlexEnd
        } else {
            JustifyContent::FlexStart
        };

        let mut query1 = query.q1();
        let mut slider_parent_style = query1.single_mut();

        slider_parent_style.justify_content = new_border_style;

        let new_slider_borler = if is_enabled {
            materials.border_enabled.clone()
        } else {
            materials.border_disabled.clone()
        };

        let mut query2 = query.q2();
        let mut material_border = query2.single_mut();

        *material_border = new_slider_borler;

        let slider_color = if is_enabled {
            materials.slider_enabled.clone()
        } else {
            materials.slider_disabled.clone()
        };

        let mut query3 = query.q3();
        let mut material_bofy = query3.single_mut();

        *material_bofy = slider_color;
    }
}

fn initial_draw(mut commands: Commands, materials: Res<ToggleMaterials>) {
    let root_size = (Val::Px(40.0), Val::Px(20.0));
    let border_width = Val::Px(1.0);
    let toggle_padding = Val::Px(3.0);
    let slider_width = Val::Px(16.0);
    let initial_toggle_state = ToggleState(false);
    commands
        // root: border
        .spawn_bundle(NodeBundle {
            style: Style {
                size: Size::new(root_size.0, root_size.1),
                padding: Rect::all(border_width),
                ..Default::default()
            },
            material: materials.border_enabled.clone(),
            ..Default::default()
        })
        .insert(initial_toggle_state)
        .with_children(|parent| {
            // root: background
            parent
                .spawn_bundle(NodeBundle {
                    style: Style {
                        size: Size::new(Val::Percent(100.0), Val::Percent(100.0)),
                        padding: Rect::all(toggle_padding),
                        justify_content: JustifyContent::FlexStart,
                        ..Default::default()
                    },
                    material: materials.bg.clone(),
                    ..Default::default()
                })
                .insert(SliderParent)
                .with_children(|parent| {
                    // toggle slider: border
                    parent
                        .spawn_bundle(NodeBundle {
                            style: Style {
                                size: Size::new(slider_width, Val::Percent(100.0)),
                                padding: Rect::all(border_width),
                                ..Default::default()
                            },
                            material: materials.border_disabled.clone(),
                            ..Default::default()
                        })
                        .insert(SliderBorder)
                        .with_children(|parent| {
                            // toggle slider: body
                            parent
                                .spawn_bundle(NodeBundle {
                                    style: Style {
                                        size: Size::new(Val::Percent(100.0), Val::Percent(100.0)),
                                        ..Default::default()
                                    },
                                    material: materials.slider_disabled.clone(),
                                    ..Default::default()
                                })
                                .insert(SliderBody);
                        });
                });
        });
}
