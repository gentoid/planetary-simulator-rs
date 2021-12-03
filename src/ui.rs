use std::f32::consts::PI;

use bevy::prelude::*;
use bevy_prototype_lyon::prelude::*;

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
            slider_enabled: materials.add(Color::rgb(0.15, 0.15, 0.15).into()),
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
        QueryState<(&mut Transform, &mut Handle<ColorMaterial>), With<SliderBorder>>,
        QueryState<(&mut Handle<ColorMaterial>), With<SliderBody>>,
    )>,
) {
    if mouse_click.pressed(MouseButton::Left) {
        let mut query0 = query.q0();
        let mut toggle_state = query0.single_mut();

        let is_enabled = !toggle_state.0;
        toggle_state.0 = is_enabled;

        let new_border_translation = if is_enabled {
            Vec3::new(12.0, 0.0, 0.0)
        } else {
            Vec3::new(0.0, 0.0, 0.0)
        };

        let new_slider_borler = if is_enabled {
            materials.border_enabled.clone()
        } else {
            materials.border_disabled.clone()
        };

        {
            let mut query1 = query.q1();
            let (mut transform_border, mut material_border) = query1.single_mut();

            transform_border.translation = new_border_translation;
            *material_border = new_slider_borler;
        }

        let slider_color = if is_enabled {
            materials.slider_enabled.clone()
        } else {
            materials.slider_disabled.clone()
        };

        {
            let mut query2 = query.q2();
            let mut material_bofy = query2.single_mut();

            *material_bofy = slider_color;
        }
    }
}

fn initial_draw(mut commands: Commands, materials: Res<ToggleMaterials>) {
    // let mut builder = PathBuilder::new();
    // builder.move_to(Vec2::new(10.0, 0.0));
    // builder.line_to(Vec2::new(20.0, 0.0));
    // builder.arc(Vec2::new(5.0, 5.0), Vec2::new(5.0, 0.0), PI / 2.0, -PI);
    // let outline_path = builder.build();
    // let mut ui_bundle = GeometryBuilder::build_ui_as(
    //     &outline_path.0,
    //     DrawMode::Stroke(StrokeMode::new(Color::DARK_GREEN, 1.0)),
    //     Style::default(),
    // );

    // ui_bundle.transform.translation = Vec3::new(0.0, 0.0, 0.0);
    // commands.spawn_bundle(ui_bundle);

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
                        ..Default::default()
                    },
                    material: materials.bg.clone(),
                    ..Default::default()
                })
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

// PathBuilder;

// fn toggle_system(
//     toggle_materials: Res<ToggleMaterials>,
//     mut interaction_query: Query<
//         (&Interaction, &mut Handle<ColorMaterial>, &Children),
//         (Changed<Interaction>, With<Button>),
//     >,
//     mut text_query: Query<&mut Text>,
// ) {
// }

// fn my_system(mut commands: Commands) {
//     let button = shapes::Rectangle {
//         extents: Vec2::new(150.0, 50.0),
//         origin: RectangleOrigin::TopLeft,
//     };
//     commands.spawn_bundle(GeometryBuilder::build_ui_as(
//         &button,
//         DrawMode::Fill(FillMode::color(Color::ORANGE_RED)),
//         Style::default(),
//     ));
// }

// pub struct UiShapeBundle {
//     pub node: Node,
//     pub style: Style,
//     pub path: Path,
//     pub mode: DrawMode,
//     pub mesh: Handle<Mesh>,
//     pub draw: Draw,
//     pub visible: Visible,
//     pub render_pipelines: RenderPipelines,
//     pub transform: Transform,
//     pub global_transform: GlobalTransform,
// }

// pub fn build_ui_as(shape: &impl Geometry, mode: DrawMode, style: Style) -> UiShapeBundle {
//     Self::new().add(shape).build_ui(mode, style)
// }
