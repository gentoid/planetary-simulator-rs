use std::f32::consts::PI;

use bevy::prelude::*;
use bevy_prototype_lyon::prelude::*;

pub struct TogglePlugin;

impl Plugin for TogglePlugin {
    fn build(&self, app: &mut App) {
        app.init_resource::<ToggleMaterials>()
            .add_startup_system(draw.system());
    }
}

struct ToggleMaterials {
    enabled: Handle<ColorMaterial>,
    disabled: Handle<ColorMaterial>,
    border: Handle<ColorMaterial>,
    bg: Handle<ColorMaterial>,
}

impl FromWorld for ToggleMaterials {
    fn from_world(world: &mut World) -> Self {
        let mut materials = world.get_resource_mut::<Assets<ColorMaterial>>().unwrap();
        ToggleMaterials {
            enabled: materials.add(Color::rgb(0.15, 0.15, 0.15).into()),
            disabled: materials.add(Color::rgb(0.25, 0.25, 0.25).into()),
            border: materials.add(Color::DARK_GREEN.into()),
            bg: materials.add(Color::BLACK.into()),
            // pressed: materials.add(Color::rgb(0.35, 0.75, 0.35).into()),
        }
    }
}

fn draw(mut commands: Commands, materials: Res<ToggleMaterials>) {
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

    commands
        // root: border
        .spawn_bundle(NodeBundle {
            style: Style {
                size: Size::new(Val::Px(25.0), Val::Px(20.0)),
                ..Default::default()
            },
            material: materials.border.clone(),
            ..Default::default()
        })
        .with_children(|parent| {
            // root: background
            parent
                .spawn_bundle(NodeBundle {
                    style: Style {
                        size: Size::new(Val::Px(23.0), Val::Px(18.0)),
                        ..Default::default()
                    },
                    material: materials.bg.clone(),
                    ..Default::default()
                })
                .with_children(|parent| {
                    // toggle slider: border
                    parent.spawn_bundle(NodeBundle {
                        style: Style {
                            size: Size::new(Val::Px(21.0), Val::Px(16.0)),
                            ..Default::default()
                        },
                        material: materials.border.clone(),
                        ..Default::default()
                    }).with_children(|parent| {
                        // toggle: body
                        parent.spawn_bundle(NodeBundle {
                            style: Style {
                                size: Size::new(Val::Px(19.0), Val::Px(14.0)),
                                ..Default::default()
                            },
                            material: materials.disabled.clone(),
                            ..Default::default()
                        });
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
