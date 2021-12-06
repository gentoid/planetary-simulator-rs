use bevy::{
    ecs::query::{FilterFetch, QueryEntityError, QueryItem, WorldQuery},
    prelude::*,
};

pub struct ToggleSwitchPlugin;

impl Plugin for ToggleSwitchPlugin {
    fn build(&self, app: &mut App) {
        app.init_resource::<Materials>().add_system(toggle.system());
    }
}

#[derive(Component)]
struct ToggleState(bool);

#[derive(Component)]
struct SliderKeeper;

#[derive(Component)]
struct ToggleSlider;

#[derive(Component)]
struct SliderBody;

pub struct Materials {
    slider_enabled: Handle<ColorMaterial>,
    slider_disabled: Handle<ColorMaterial>,
    border_disabled: Handle<ColorMaterial>,
    border_enabled: Handle<ColorMaterial>,
    bg: Handle<ColorMaterial>,
}

impl FromWorld for Materials {
    fn from_world(world: &mut World) -> Self {
        let mut materials = world.get_resource_mut::<Assets<ColorMaterial>>().unwrap();
        Materials {
            slider_enabled: materials.add(Color::rgb(0.0, 0.7, 0.0).into()),
            slider_disabled: materials.add(Color::rgb(0.25, 0.25, 0.25).into()),
            border_disabled: materials.add(Color::rgb(0.2, 0.2, 0.2).into()),
            border_enabled: materials.add(Color::DARK_GREEN.into()),
            bg: materials.add(Color::BLACK.into()),
        }
    }
}

pub fn draw<'a>(
    initial_state: bool,
    materials: &'a Res<Materials>,
) -> impl Fn(&mut ChildBuilder) + 'a {
    return move |parent| {
        let root_size = (Val::Px(40.0), Val::Px(20.0));
        let border_width = Val::Px(1.0);
        let toggle_padding = Val::Px(3.0);
        let slider_width = Val::Px(16.0);

        parent
            // root: border
            .spawn_bundle(NodeBundle {
                style: Style {
                    size: Size::new(root_size.0, root_size.1),
                    padding: Rect::all(border_width),
                    position: Rect {
                        left: Val::Px(0.0),
                        bottom: Val::Px(0.0),
                        right: Val::Undefined,
                        top: Val::Undefined,
                    },
                    ..Default::default()
                },
                material: materials.border_enabled.clone(),
                ..Default::default()
            })
            .insert(ToggleState(initial_state))
            .with_children(|parent| {
                // root: background
                parent
                    .spawn_bundle(NodeBundle {
                        style: Style {
                            size: Size::new(Val::Percent(100.0), Val::Percent(100.0)),
                            padding: Rect::all(toggle_padding),
                            justify_content: slider_keeper_justify_content(initial_state),
                            ..Default::default()
                        },
                        material: materials.bg.clone(),
                        ..Default::default()
                    })
                    .insert(SliderKeeper)
                    .with_children(|parent| {
                        // toggle slider: border
                        parent
                            .spawn_bundle(NodeBundle {
                                style: Style {
                                    size: Size::new(slider_width, Val::Percent(100.0)),
                                    padding: Rect::all(border_width),
                                    ..Default::default()
                                },
                                material: slider_border_color(initial_state, &materials),
                                ..Default::default()
                            })
                            .insert(ToggleSlider)
                            .with_children(|parent| {
                                // toggle slider: body
                                parent
                                    .spawn_bundle(NodeBundle {
                                        style: Style {
                                            size: Size::new(
                                                Val::Percent(100.0),
                                                Val::Percent(100.0),
                                            ),
                                            ..Default::default()
                                        },
                                        material: slider_body_color(initial_state, &materials),
                                        ..Default::default()
                                    })
                                    .insert(SliderBody);
                            });
                    });
            });
    };
}

fn toggle(
    mouse_click: Res<Input<MouseButton>>,
    windows: Res<Windows>,
    materials: Res<Materials>,
    mut states_query: Query<
        (&mut ToggleState, &Style, &GlobalTransform, &Children),
        Without<SliderKeeper>,
    >,
    mut slider_keepers_query: Query<(&mut Style, &Children), With<SliderKeeper>>,
    mut sliders_query: Query<
        (&mut Handle<ColorMaterial>, &Children),
        (With<ToggleSlider>, Without<SliderBody>),
    >,
    mut slider_body_query: Query<&mut Handle<ColorMaterial>, With<SliderBody>>,
) {
    if !mouse_click.just_pressed(MouseButton::Left) {
        return;
    }

    let primary_window = windows.get_primary().unwrap();

    let cursor_position = match primary_window.cursor_position() {
        None => return,
        Some(position) => position,
    };

    for (mut toggle_state, style, global_transform, children) in states_query.iter_mut() {
        if !does_cursor_hover_element(style.size, global_transform, cursor_position) {
            continue;
        }

        let is_enabled = !toggle_state.0;
        toggle_state.0 = is_enabled;

        let rr = first_child(slider_keepers_query)(children)
            .map(update_slider_keeper(is_enabled))
            .and_then(first_child(sliders_query))
            .map(update_slider_border(&is_enabled, &materials))
            .and_then(first_child(slider_body_query))
            .map(update_clider_body(&is_enabled, &materials));

        if let Err(err) = rr {
            warn!("UI::ToggleSwitch error: {:?}", err);
        }

        return;
    }
}

fn first_child<'w, 's, Q: WorldQuery, F: WorldQuery>(
    query: Query<'w, 's, Q, F>,
) -> impl FnOnce(&Children) -> Result<QueryItem<Q>, QueryEntityError>
where
    F::Fetch: FilterFetch,
{
    move |children| children
    .first()
    .ok_or(QueryEntityError::NoSuchEntity)
    .and_then(|child| query.get_mut(*child))
}

fn does_cursor_hover_element(
    size: Size<Val>,
    global_transform: &GlobalTransform,
    cursor_position: Vec2,
) -> bool {
    match (size.width, size.height) {
        (Val::Px(width), Val::Px(height)) => {
            let center = global_transform.translation.truncate();
            let half_size = Vec2::new(width, height) / 2.0;

            let left_bottom_corner = center - half_size;
            let right_top_corner = center + half_size;

            cursor_position.cmpge(left_bottom_corner).all()
                && cursor_position.cmple(right_top_corner).all()
        }
        _ => false,
    }
}

fn update_slider_keeper<'a>(
    is_enabled: bool,
) -> impl FnOnce((Mut<Style>, &'a Children)) -> &'a Children {
    move |(mut style, children)| {
        style.justify_content = slider_keeper_justify_content(is_enabled);

        children
    }
}

fn slider_keeper_justify_content(is_enabled: bool) -> JustifyContent {
    if is_enabled {
        JustifyContent::FlexEnd
    } else {
        JustifyContent::FlexStart
    }
}

fn update_slider_border<'a>(
    is_enabled: &'a bool,
    materials: &'a Res<Materials>,
) -> impl FnOnce((Mut<Handle<ColorMaterial>>, &'a Children)) -> &'a Children + 'a {
    move |(mut border_color, children)| {
        *border_color = slider_border_color(*is_enabled, &materials);

        children
    }
}

fn slider_border_color(is_enabled: bool, materials: &Res<Materials>) -> Handle<ColorMaterial> {
    if is_enabled {
        materials.border_enabled.clone()
    } else {
        materials.border_disabled.clone()
    }
}

fn update_clider_body<'a>(
    is_enabled: &'a bool,
    materials: &'a Res<Materials>,
) -> impl FnOnce(Mut<Handle<ColorMaterial>>) + 'a {
    move |mut body_color| {
        *body_color = slider_body_color(*is_enabled, &materials);
    }
}

fn slider_body_color(is_enabled: bool, materials: &Res<Materials>) -> Handle<ColorMaterial> {
    if is_enabled {
        materials.slider_enabled.clone()
    } else {
        materials.slider_disabled.clone()
    }
}
