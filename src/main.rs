#![windows_subsystem = "windows"]

use bevy::{core::FixedTimestep, prelude::*};

// Consts
const SNAKE_HEAD_COLOR: Color = Color::rgb(0.7, 0.7, 0.7);
const SNAKE_SEGMENT_COLOR: Color = Color::rgb(0.3, 0.3, 0.3);

const FOOD_COLOR: Color = Color::rgb(1.0, 0.0, 1.0);

const ARENA_WIDTH: u32 = 30;
const ARENA_HEIGHT: u32 = 30;

#[derive(PartialEq, Clone, Copy)]
enum Direction {
    Left,
    Up,
    Right,
    Down,
}

impl Direction {
    fn opposite(self) -> Self {
        match self {
            Self::Left => Self::Right,
            Self::Up => Self::Down,
            Self::Right => Self::Left,
            Self::Down => Self::Up,
        }
    }
}

// Components
#[derive(Component)]
struct SnakeHead {
    direction: Direction,
}

#[derive(Component)]
struct SnakeSegment;

#[derive(Component)]
struct Food;

#[derive(Component, Clone, Copy, PartialEq, Eq)]
struct Position {
    x: i32,
    y: i32,
}

#[derive(Component)]
struct Size {
    width: f32,
    height: f32,
}

impl Size {
    fn square(x: f32) -> Self {
        Self {
            width: x,
            height: x,
        }
    }
}

// Resources
#[derive(Default, Deref, DerefMut)]
struct SnakeSegments(Vec<Entity>);

#[derive(Default, Deref, DerefMut)]
struct LastTailPosition(Option<Position>);

// Events
struct GrowthEvent;

struct GameOverEvent;

// Systems
fn setup_camera(mut commands: Commands) {
    commands.spawn_bundle(OrthographicCameraBundle::new_2d());
}

fn spawn_snake(mut commands: Commands, mut segments: ResMut<SnakeSegments>) {
    *segments = SnakeSegments(vec![
        commands
            .spawn_bundle(SpriteBundle {
                sprite: Sprite {
                    color: SNAKE_HEAD_COLOR,
                    ..default()
                },
                transform: {
                    let mut t = Transform {
                        scale: Vec3::new(10.0, 10.0, 10.0),
                        ..default()
                    };
                    t.translation.z = 2.0;
                    t
                },
                ..default()
            })
            .insert(SnakeHead {
                direction: Direction::Right,
            })
            .insert(SnakeSegment)
            .insert(Position { x: 3, y: 3 })
            .insert(Size::square(0.8))
            .id(),
        spawn_segment(commands, Position { x: 3, y: 2 }),
    ]);
}

fn snake_movement_input(keyboard_input: Res<Input<KeyCode>>, mut q: Query<&mut SnakeHead>) {
    let mut head = q.single_mut();

    let dir = if keyboard_input.pressed(KeyCode::Left) {
        Direction::Left
    } else if keyboard_input.pressed(KeyCode::Right) {
        Direction::Right
    } else if keyboard_input.pressed(KeyCode::Down) {
        Direction::Down
    } else if keyboard_input.pressed(KeyCode::Up) {
        Direction::Up
    } else {
        head.direction
    };

    if dir != head.direction.opposite() {
        head.direction = dir;
    }
}

fn snake_movement(
    segments: ResMut<SnakeSegments>,
    head: Query<(Entity, &SnakeHead)>,
    mut positions: Query<&mut Position, With<SnakeSegment>>,
    mut last_tail_position: ResMut<LastTailPosition>,
    mut game_over_writer: EventWriter<GameOverEvent>,
) {
    let (head_entity, head) = head.single();
    let segment_positions = segments
        .iter()
        .map(|&e| *positions.get(e).unwrap())
        .collect::<Vec<_>>();
    let mut head_position = positions.get_mut(head_entity).unwrap();

    match &head.direction {
        Direction::Left => {
            head_position.x -= 1;
        }
        Direction::Right => {
            head_position.x += 1;
        }
        Direction::Down => {
            head_position.y -= 1;
        }
        Direction::Up => {
            head_position.y += 1;
        }
    }

    if head_position.x < 0
        || head_position.y < 0
        || head_position.x as u32 >= ARENA_WIDTH
        || head_position.y as u32 >= ARENA_HEIGHT
        || segment_positions.contains(&head_position)
    {
        game_over_writer.send(GameOverEvent);
    }

    segment_positions
        .iter()
        .zip(segments.iter().skip(1))
        .for_each(|(&pos, &segment)| {
            *positions.get_mut(segment).unwrap() = pos;
        });

    *last_tail_position = LastTailPosition(Some(*segment_positions.last().unwrap()));
}

fn snake_eating(
    mut commands: Commands,
    mut growth_writer: EventWriter<GrowthEvent>,
    food_positions: Query<(Entity, &Position), With<Food>>,
    head_position: Query<&Position, With<SnakeHead>>,
) {
    let head_pos = head_position.single();

    for (ent, food_pos) in food_positions.iter() {
        if food_pos == head_pos {
            commands.entity(ent).despawn();
            growth_writer.send(GrowthEvent);
        }
    }
}

fn snake_growth(
    commands: Commands,
    last_tail_position: Res<LastTailPosition>,
    mut segments: ResMut<SnakeSegments>,
    mut growth_reader: EventReader<GrowthEvent>,
) {
    if growth_reader.iter().next().is_some() {
        segments.push(spawn_segment(commands, last_tail_position.0.unwrap()));
    }
}

fn food_spawner(mut commands: Commands) {
    commands
        .spawn_bundle(SpriteBundle {
            sprite: Sprite {
                color: FOOD_COLOR,
                ..default()
            },
            ..default()
        })
        .insert(Food)
        .insert(Position {
            x: (fastrand::f32() * ARENA_WIDTH as f32) as i32,
            y: (fastrand::f32() * ARENA_HEIGHT as f32) as i32,
        })
        .insert(Size::square(0.8));
}

fn game_over(
    mut commands: Commands,
    mut reader: EventReader<GameOverEvent>,
    segments_res: ResMut<SnakeSegments>,
    food: Query<Entity, With<Food>>,
    segments: Query<Entity, With<SnakeSegment>>,
) {
    if reader.iter().next().is_some() {
        food.iter().chain(segments.iter()).for_each(|e| {
            commands.entity(e).despawn();
        });
        spawn_snake(commands, segments_res);
    }
}

fn size_scaling(windows: Res<Windows>, mut q: Query<(&Size, &mut Transform)>) {
    let window = windows.get_primary().unwrap();

    for (sprite_size, mut transform) in q.iter_mut() {
        transform.scale = Vec3::new(
            sprite_size.width / ARENA_WIDTH as f32 * window.width(),
            sprite_size.height / ARENA_HEIGHT as f32 * window.height(),
            1.0,
        );
    }
}

fn position_translation(windows: Res<Windows>, mut q: Query<(&Position, &mut Transform)>) {
    fn convert(pos: f32, bound_window: f32, bound_game: f32) -> f32 {
        let tile_size = bound_window / bound_game;
        pos / bound_game * bound_window - (bound_window / 2.) + (tile_size / 2.)
    }

    let window = windows.get_primary().unwrap();
    for (pos, mut transform) in q.iter_mut() {
        transform.translation = Vec3::new(
            convert(pos.x as f32, window.width(), ARENA_WIDTH as f32),
            convert(pos.y as f32, window.height(), ARENA_HEIGHT as f32),
            transform.translation[2],
        )
    }
}

fn spawn_segment(mut commands: Commands, position: Position) -> Entity {
    commands
        .spawn_bundle(SpriteBundle {
            sprite: Sprite {
                color: SNAKE_SEGMENT_COLOR,
                ..default()
            },
            transform: {
                let mut t = Transform { ..default() };
                t.translation.z = 2.0;
                t
            },
            ..default()
        })
        .insert(SnakeSegment)
        .insert(position)
        .insert(Size::square(0.7))
        .id()
}
fn main() {
    App::new()
        .insert_resource(WindowDescriptor {
            title: String::from("Snake"),
            width: 500.0,
            height: 500.0,
            ..default()
        })
        .insert_resource(ClearColor(Color::rgb(0.04, 0.04, 0.04)))
        .init_resource::<SnakeSegments>()
        .init_resource::<LastTailPosition>()
        .add_event::<GrowthEvent>()
        .add_event::<GameOverEvent>()
        .add_startup_system(setup_camera)
        .add_startup_system(spawn_snake)
        .add_system(snake_movement_input.before(snake_movement))
        .add_system(game_over.after(snake_movement))
        .add_system_set(
            SystemSet::new()
                .with_run_criteria(FixedTimestep::step(0.1))
                .with_system(snake_movement)
                .with_system(snake_eating.after(snake_movement))
                .with_system(snake_growth.after(snake_eating)),
        )
        .add_system_set(
            SystemSet::new()
                .with_run_criteria(FixedTimestep::step(1.0))
                .with_system(food_spawner),
        )
        .add_system_set_to_stage(
            CoreStage::PostUpdate,
            SystemSet::new()
                .with_system(position_translation)
                .with_system(size_scaling),
        )
        .add_plugins(DefaultPlugins)
        .run();
}
