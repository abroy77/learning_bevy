use bevy::math::bounding::{Aabb2d, BoundingCircle, BoundingVolume, IntersectsVolume};
use bevy::prelude::*;
use bevy::sprite::MaterialMesh2dBundle;
use rand::random;
const BALL_RADIUS: f32 = 5.;
const PADDLE_WIDTH: f32 = 10.;
const PADDLE_HEIGHT: f32 = 50.;
const GUTTER_HEIGHT: f32 = 20.;
const PADDLE_SPEED: f32 = 5.;

#[derive(Component)]
struct Shape(Vec2);

#[derive(Component)]
struct Velocity(Vec2);

#[derive(Component)]
struct Paddle;

#[derive(Bundle)]
struct PaddleBundle {
    paddle: Paddle,
    position: Position,
    shape: Shape,
    velocity: Velocity,
}

impl PaddleBundle {
    fn new(x: f32, y: f32) -> Self {
        PaddleBundle {
            paddle: Paddle,
            position: Position(Vec2::new(x, y)),
            velocity: Velocity(Vec2::new(0., 0.)),
            shape: Shape(Vec2::new(PADDLE_WIDTH, PADDLE_HEIGHT)),
        }
    }
}

#[derive(Component)]
struct Position(Vec2);

#[derive(Component)]
struct Ball;

#[derive(Bundle)]
struct BallBundle {
    ball: Ball,
    position: Position,
    velocity: Velocity,
    shape: Shape,
}

impl BallBundle {
    fn new(v_x: f32, v_y: f32) -> Self {
        BallBundle {
            ball: Ball,
            position: Position(Vec2::new(0., 0.)),
            velocity: Velocity(Vec2::new(v_x, v_y)),
            shape: Shape(Vec2::new(BALL_RADIUS, BALL_RADIUS)),
        }
    }
}

#[derive(Debug, PartialEq, Eq, Copy, Clone)]
enum Collision {
    Top,
    Bottom,
    Left,
    Right,
}

enum Scorer {
    Player,
    Ai,
}

#[derive(Event)]
struct Scored(Scorer);

#[derive(Resource, Default)]
struct Score {
    player: u32,
    ai: u32,
}

#[derive(Component)]
struct Player;
#[derive(Component)]
struct Ai;

#[derive(Component)]
struct Gutter;

#[derive(Bundle)]
struct GutterBundle {
    gutter: Gutter,
    position: Position,
    shape: Shape,
}

impl GutterBundle {
    fn new(x: f32, y: f32, width: f32) -> Self {
        GutterBundle {
            gutter: Gutter,
            position: Position(Vec2::new(x, y)),
            shape: Shape(Vec2::new(width, GUTTER_HEIGHT)),
        }
    }
}

#[derive(Component)]
struct PlayerScoreboard;
#[derive(Component)]
struct AiScoreboard;

fn main() {
    App::new()
        .add_plugins(DefaultPlugins)
        .init_resource::<Score>()
        .add_event::<Scored>()
        .add_systems(
            Startup,
            (spawn_camera, spawn_ball, spawn_paddles, spawn_gutters, spawn_scoreboard),
        )
        .add_systems(
            Update,
            (
                move_ball,
                // Add our projection system to run after
                // we move our ball so we are not reading
                // movement one frame behind
                project_positions.after(move_ball),
                handle_collisions.after(move_ball),
                handle_player_input.after(move_ball),
                move_paddles.after(handle_player_input),
                detect_scoring.after(move_ball),
                reset_ball.after(detect_scoring),
                update_score.after(detect_scoring),
                update_scoreboard.after(update_score),
            ),
        )
        .run();
}

fn spawn_ball(
    mut commands: Commands,
    mut meshes: ResMut<Assets<Mesh>>,
    mut materials: ResMut<Assets<ColorMaterial>>,
) {
    println!("Spawning ball");

    let shape = Mesh::from(Circle::new(BALL_RADIUS));
    let material = ColorMaterial::from_color(Color::srgb_u8(50, 100, 200));

    // `Assets::add` will load these into memory and return a
    // `Handle` (an ID) to these assets. When all references
    // to this `Handle` are cleaned up the asset is cleaned up.

    let mesh_handle = meshes.add(shape);
    let material_handle = materials.add(material);

    // Here we are using `spawn` instead of `spawn_empty`
    // followed by an `insert`. They mean the same thing,
    // letting us spawn many components on a new entity at once.

    commands.spawn((
        BallBundle::new(5., 0.),
        MaterialMesh2dBundle {
            mesh: mesh_handle.into(),
            material: material_handle,
            ..default()
        },
    ));
}

fn spawn_gutters(
    mut commands: Commands,
    mut meshes: ResMut<Assets<Mesh>>,
    mut materials: ResMut<Assets<ColorMaterial>>,
    window: Query<&Window>,
) {
    println!("Spawning gutters");

    if let Ok(window) = window.get_single() {
        let window_width = window.resolution.width();
        let window_height = window.resolution.height();

        let top_gutter_y = window_height / 2. - GUTTER_HEIGHT / 2.;
        let bottom_gutter_y = -window_height / 2. + GUTTER_HEIGHT / 2.;

        let top_gutter = GutterBundle::new(0., top_gutter_y, window_width);
        let bottom_gutter = GutterBundle::new(0., bottom_gutter_y, window_width);

        let shape = Mesh::from(Rectangle::new(window_width, GUTTER_HEIGHT));
        let material = ColorMaterial::from_color(Color::srgb_u8(255, 255, 255));

        let mesh_handle = meshes.add(shape);
        let material_handle = materials.add(material);

        commands.spawn((
            top_gutter,
            MaterialMesh2dBundle {
                mesh: mesh_handle.clone().into(),
                material: material_handle.clone(),
                ..default()
            },
        ));

        commands.spawn((
            bottom_gutter,
            MaterialMesh2dBundle {
                mesh: mesh_handle.into(),
                material: material_handle,
                ..default()
            },
        ));
    }
}

fn spawn_paddles(
    mut commands: Commands,
    mut meshes: ResMut<Assets<Mesh>>,
    mut materials: ResMut<Assets<ColorMaterial>>,
    window: Query<&Window>,
) {
    println!("Spawning paddle");
    // get the window
    if let Ok(window) = window.get_single() {
        let window_width = window.resolution.width();
        let padding = 50.;
        let right_paddle_x = window_width / 2. - padding;
        let left_paddle_x = -window_width / 2. + padding;

        // make the meshes and materials

        let shape = Mesh::from(Rectangle::new(PADDLE_WIDTH, PADDLE_HEIGHT));
        let material = ColorMaterial::from_color(Color::srgb_u8(200, 100, 50));

        // add the meshes and materials to the asset manager
        let mesh_handle = meshes.add(shape);
        let material_handle = materials.add(material);
        commands.spawn((
            Player,
            PaddleBundle::new(left_paddle_x, 0.),
            MaterialMesh2dBundle {
                mesh: mesh_handle.clone().into(),
                material: material_handle.clone(),
                ..default()
            },
        ));

        commands.spawn((
            Ai,
            PaddleBundle::new(right_paddle_x, 0.),
            MaterialMesh2dBundle {
                mesh: mesh_handle.into(),
                material: material_handle.into(),
                ..default()
            },
        ));
    }
}

fn project_positions(mut positionables: Query<(&mut Transform, &Position)>) {
    // Our position is `Vec2` but a translation is `Vec3`
    // so we extend our `Vec2` into one by adding a `z`
    // value of 0

    for (mut transform, position) in &mut positionables {
        transform.translation = position.0.extend(0.);
    }
}

fn spawn_camera(mut commands: Commands) {
    println!("Spawning camera");
    commands.spawn_empty().insert(Camera2dBundle::default());
}

fn move_ball(
    // Give me all positions that also contain a `Ball` component
    mut ball: Query<(&mut Position, &Velocity), With<Ball>>,
) {
    // this is different from the tutorial
    // tutorial is outdated
    if let Ok((mut position, velocity)) = ball.get_single_mut() {
        position.0.x += velocity.0.x;
        position.0.y += velocity.0.y;
    }
}

fn collide_with_side(ball: BoundingCircle, wall: Aabb2d) -> Option<Collision> {
    if !ball.intersects(&wall) {
        return None;
    }

    let closest_point = wall.closest_point(ball.center());
    let offset = ball.center() - closest_point;

    let side = if offset.x.abs() > offset.y.abs() {
        if offset.x > 0. {
            Collision::Left
        } else {
            Collision::Right
        }
    } else {
        if offset.y > 0. {
            Collision::Bottom
        } else {
            Collision::Top
        }
    };

    Some(side)
}

fn handle_collisions(
    mut ball: Query<(&mut Velocity, &Position, &Shape), With<Ball>>,
    others: Query<(&Position, &Shape), Without<Ball>>,
) {
    // get the single ball
    if let Ok((mut ball_velocity, ball_position, ball_shape)) = ball.get_single_mut() {
        let ball_circle = BoundingCircle::new(ball_position.0, ball_shape.0.x);

        for (position, shape) in &others {
            let other_rect = Aabb2d::new(position.0, shape.0 / 2.);
            if let Some(collision) = collide_with_side(ball_circle, other_rect) {
                match collision {
                    Collision::Top | Collision::Bottom => {
                        ball_velocity.0.y *= -1.;
                    }
                    Collision::Left | Collision::Right => {
                        ball_velocity.0.x *= -1.;
                    }
                }
            }
        }
    }
}

fn handle_player_input(
    keyboard_input: Res<ButtonInput<KeyCode>>,
    mut player_paddle: Query<&mut Velocity, With<Player>>,
    mut ai_paddle: Query<&mut Velocity, (With<Ai>, Without<Player>)>,
) {
    if let Ok(mut velocity) = player_paddle.get_single_mut() {
        if keyboard_input.pressed(KeyCode::KeyY) {
            velocity.0.y = PADDLE_SPEED;
        } else if keyboard_input.pressed(KeyCode::KeyN) {
            velocity.0.y = -PADDLE_SPEED;
        } else {
            velocity.0.y = 0.;
        };
    }
    
    if let Ok(mut velocity) = ai_paddle.get_single_mut() {
        if keyboard_input.pressed(KeyCode::KeyW) {
            velocity.0.y = PADDLE_SPEED;
        } else if keyboard_input.pressed(KeyCode::KeyX) {
            velocity.0.y = -PADDLE_SPEED;
        } else {
            velocity.0.y = 0.;
        };
    }
}

fn move_paddles(
    mut paddles: Query<(&mut Position, &Velocity), With<Paddle>>,
    window: Query<&Window>,
) {
    if let Ok(window) = window.get_single() {
        let window_height = window.resolution.height();
        for (mut position, velocity) in &mut paddles {
            position.0.y += velocity.0.y;
            position.0.y = position.0.y.max(-window_height / 2. + PADDLE_HEIGHT / 2.);
            position.0.y = position.0.y.min(window_height / 2. - PADDLE_HEIGHT / 2.);
        }
    }
}

fn detect_scoring(
    ball: Query<&Position, With<Ball>>,
    window: Query<&Window>,
    mut events: EventWriter<Scored>,
) {
    // get the window
    if let Ok(window) = window.get_single() {
        let window_width = window.resolution.width();

        // get the ball
        if let Ok(ball_position) = ball.get_single() {
            if ball_position.0.x > window_width / 2. {
                events.send(Scored(Scorer::Player));
            } else if ball_position.0.x < -window_width / 2. {
                events.send(Scored(Scorer::Ai));
            }
        } else {
            eprintln!("No ball found in the scene.");
        }
    } else {
        eprintln!("No window found in the scene.");
    }
}

fn update_score(mut score: ResMut<Score>, mut scored_events: EventReader<Scored>) {
    for event in scored_events.read() {
        match event.0 {
            Scorer::Player => score.player += 1,
            Scorer::Ai => score.ai += 1,
        }
    }
    println!(" Score: Player: {} \n     Ai: {}", score.player, score.ai);
}

fn reset_ball(
    mut ball: Query<(&mut Position, &mut Velocity), With<Ball>>,
    mut events: EventReader<Scored>,
) {
    for event in events.read() {
        if let Ok((mut position, mut velocity)) = ball.get_single_mut() {
            position.0 = Vec2::new(0., 0.);
            let random_v_y = (random::<f32>() - 0.5) * 3.;
            let random_v_y = random_v_y + random_v_y.signum() * 4.;

            let random_v_x_mag = 4. + random::<f32>() * 3.;

            // get the current score
            let x_dir = match event.0 {
                Scorer::Player => -1.,
                Scorer::Ai => 1.,
            };

            velocity.0 = Vec2::new(x_dir * random_v_x_mag, random_v_y);
        }
    }
}

fn spawn_scoreboard(mut commands: Commands) {
    println!("Spawning Scoreboard");

    commands.spawn((
        TextBundle::from_section(
            "0",
            TextStyle {
                font_size: 50.,
                color: Color::WHITE,
                ..default()
            },
        )
        .with_text_justify(JustifyText::Center)
        .with_style(Style {
            position_type: PositionType::Absolute,
            top: Val::Px(20.0),
            left: Val::Px(10.0),
            ..default()
        }),
        PlayerScoreboard,
    ));


    commands.spawn((
        TextBundle::from_section(
            "0",
            TextStyle {
                font_size: 50.,
                color: Color::WHITE,
                ..default()
            },
        )
        .with_text_justify(JustifyText::Center)
        .with_style(Style {
            position_type: PositionType::Absolute,
            top: Val::Px(20.0),
            left: Val::Px(40.0),
            ..default()
        }),
        AiScoreboard,
    ));
}

fn update_scoreboard(
    mut player_score: Query<&mut Text, With<PlayerScoreboard>>,
    mut ai_score: Query<&mut Text, (With<AiScoreboard>, Without<PlayerScoreboard>)>,
    score: Res<Score>,
) {
    if score.is_changed() {
        if let Ok(mut player_score) = player_score.get_single_mut() {
            player_score.sections[0].value = score.player.to_string();
        }

        if let Ok(mut ai_score) = ai_score.get_single_mut() {
            ai_score.sections[0].value = score.ai.to_string();
        }
    }
}
