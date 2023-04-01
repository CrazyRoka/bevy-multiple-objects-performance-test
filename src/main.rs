use std::time::Duration;

use bevy::{
    diagnostic::{Diagnostics, FrameTimeDiagnosticsPlugin},
    prelude::*,
};
use bevy_editor_pls::prelude::*;
use rand::{distributions::Uniform, prelude::Distribution, thread_rng, Rng};

const INITIAL_SPAWNING_RATE: u32 = 100;
const SPAWNING_RATE_STEP: u32 = 500;

fn main() {
    App::new()
        .add_plugins(DefaultPlugins)
        .add_plugin(FrameTimeDiagnosticsPlugin::default())
        .add_plugin(EditorPlugin)
        .add_startup_system(setup)
        .add_system(input_system)
        .add_system(cube_spawning_system)
        .add_system(movement_system)
        .add_system(counter_system)
        .run();
}

#[derive(Resource)]
struct MyCube {
    mesh: Handle<Mesh>,
    material: Handle<StandardMaterial>,
}

#[derive(Resource)]
struct CubesCounter {
    count: u32,
}

fn setup(
    mut commands: Commands,
    mut meshes: ResMut<Assets<Mesh>>,
    mut materials: ResMut<Assets<StandardMaterial>>,
    asset_server: Res<AssetServer>,
) {
    let my_cube = MyCube {
        mesh: meshes.add(shape::Cube::new(1.0).into()),
        material: materials.add(Color::rgb(0.8, 0.7, 0.6).into()),
    };

    commands.insert_resource(my_cube);
    commands.insert_resource(CubesCounter { count: 0 });
    commands.spawn(CubeSpawner {
        spawning_rate: INITIAL_SPAWNING_RATE,
        timer: Timer::new(Duration::from_secs(1), TimerMode::Repeating),
    });

    let text_section = move |color, value: &str| {
        TextSection::new(
            value,
            TextStyle {
                font: asset_server.load("fonts/Roboto-Regular.ttf"),
                font_size: 40.0,
                color,
            },
        )
    };

    commands.spawn((
        TextBundle::from_sections([
            text_section(Color::GREEN, "Cubes Count: "),
            text_section(Color::CYAN, ""),
            text_section(Color::GREEN, "\nSpawning Rate: "),
            text_section(Color::CYAN, ""),
            text_section(Color::GREEN, "\nFPS (raw): "),
            text_section(Color::CYAN, ""),
            text_section(Color::GREEN, "\nFPS (SMA): "),
            text_section(Color::CYAN, ""),
            text_section(Color::GREEN, "\nFPS (EMA): "),
            text_section(Color::CYAN, ""),
        ])
        .with_style(Style {
            position_type: PositionType::Absolute,
            position: UiRect {
                top: Val::Px(25.0),
                left: Val::Px(5.0),
                ..default()
            },
            ..default()
        }),
        StatsText,
    ));

    commands.spawn(Camera3dBundle {
        transform: Transform::from_xyz(-20.0, 25.0, 50.0).looking_at(Vec3::ZERO, Vec3::Y),
        ..Default::default()
    });
}

fn movement_system(mut query: Query<(&GeneratedCube, &mut Transform)>, time: Res<Time>) {
    for (cube, mut transform) in query.iter_mut() {
        transform.translation.x += time.delta_seconds() * cube.speed;

        if transform.translation.x > cube.x_range {
            transform.translation.x -= cube.x_range * 2.0;
        }
    }
}

#[derive(Component)]
struct GeneratedCube {
    speed: f32,
    x_range: f32,
}

#[derive(Component)]
struct CubeSpawner {
    spawning_rate: u32,
    timer: Timer,
}

fn cube_spawning_system(
    mut commands: Commands,
    mut query: Query<&mut CubeSpawner>,
    time: Res<Time>,
    my: Res<MyCube>,
    mut counter: ResMut<CubesCounter>,
) {
    for mut spawner in query.iter_mut() {
        spawner.timer.tick(time.delta());

        if spawner.timer.finished() {
            for _ in 0..spawner.spawning_rate {
                let between = Uniform::from(-10.0..10.0);
                let mut rng = thread_rng();
                let x = between.sample(&mut rng);
                let y = between.sample(&mut rng);
                let z = between.sample(&mut rng);

                commands.spawn((
                    PbrBundle {
                        mesh: my.mesh.clone(),
                        material: my.material.clone(),
                        transform: Transform::from_xyz(x, y, z),
                        ..Default::default()
                    },
                    GeneratedCube {
                        x_range: 20.0,
                        speed: 10.0,
                    },
                ));
                counter.count += 1;
            }
        }
    }
}

fn input_system(
    keyboard_input: Res<Input<KeyCode>>,
    mut query: Query<&mut CubeSpawner>,
    time: Res<Time>,
) {
    let mut spawner = query.single_mut();
    if keyboard_input.pressed(KeyCode::W) {
        let increase = (SPAWNING_RATE_STEP as f32) * time.delta_seconds();
        let rounded = increase.round() as u32;
        spawner.spawning_rate += rounded;
    }

    if keyboard_input.pressed(KeyCode::S) {
        let decrease = (SPAWNING_RATE_STEP as f32) * time.delta_seconds();
        let rounded = decrease.round() as u32;
        spawner.spawning_rate =
            if let Some(spawning_rate) = spawner.spawning_rate.checked_sub(rounded) {
                spawning_rate
            } else {
                0
            }
    }
}

#[derive(Component)]
struct StatsText;

fn counter_system(
    diagnostics: Res<Diagnostics>,
    counter: Res<CubesCounter>,
    spawner_query: Query<&CubeSpawner>,
    mut query: Query<&mut Text, With<StatsText>>,
) {
    let mut text = query.single_mut();

    if counter.is_changed() {
        text.sections[1].value = counter.count.to_string();
    }

    let spawner = spawner_query.single();
    text.sections[3].value = spawner.spawning_rate.to_string();

    if let Some(fps) = diagnostics.get(FrameTimeDiagnosticsPlugin::FPS) {
        if let Some(raw) = fps.value() {
            text.sections[5].value = format!("{raw:.2}");
        }
        if let Some(sma) = fps.average() {
            text.sections[7].value = format!("{sma:.2}");
        }
        if let Some(ema) = fps.smoothed() {
            text.sections[9].value = format!("{ema:.2}");
        }
    };
}
