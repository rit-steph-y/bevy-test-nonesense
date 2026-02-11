use bevy::prelude::*;

const LATEST_WINDOW_MILLS: u128 = 300;
const EARLIEST_WINDOW_MILLS: u128 = 300;
#[derive(Component, Ord, Eq, PartialEq)]
struct TimeUntilClick {
    nano: u128,
}

impl PartialOrd for TimeUntilClick{
    fn partial_cmp(&self, other: &Self) -> Option<std::cmp::Ordering> {
        return Some(self.nano.cmp(&other.nano));
    }
}

#[derive(Component)]
#[component(storage = "SparseSet")]
struct CanHit;

const MILLS_NANO_CONVER_FACTOR: u128 = 1_000_000u128;

impl TimeUntilClick {
    fn new_mills(mills: u128) -> Self {
        Self {
            nano: (mills + LATEST_WINDOW_MILLS) * MILLS_NANO_CONVER_FACTOR,
        }
    }
}

fn spawn_note(commands: &mut Commands, mills: u128) {
    commands.spawn((
        TimeUntilClick::new_mills(mills),
        Sprite::from_color(DON_COLOR, Vec2::new(80., 80.)),
    ));
}

fn update_timed(time: Res<Time>, query: Query<&mut TimeUntilClick>) {
    let nanos = time.delta().as_nanos();
    for mut timed_item in query {
        if nanos >= timed_item.nano {
            timed_item.nano = 0;
        } else {
            timed_item.nano -= nanos;
        }
    }
}

fn free_timed(mut command: Commands, query: Query<(Entity, &mut TimeUntilClick)>) {
    for (entity, time) in query {
        if time.nano == 0 {
            command.entity(entity).despawn();
        }
    }
}

fn mark_pressable(mut command: Commands, query: Query<(Entity, &TimeUntilClick), Without<CanHit>>) {
    for (entity, time) in query {
        if time.nano <= (EARLIEST_WINDOW_MILLS + LATEST_WINDOW_MILLS) * MILLS_NANO_CONVER_FACTOR{
            command.entity(entity).insert(CanHit);
        }
    }
}

fn spawn_notes(mut commands: Commands) {
    for i in 0..100{
        spawn_note(&mut commands, 1000 * i);
        spawn_note(&mut commands, 1000 * i + 200);
    }
    // spawn_note(&mut commands, 1000);
    // // spawn_note(&mut commands, 1500);
    // // spawn_note(&mut commands, 1800);
    // spawn_note(&mut commands, 3000);
}

const DON_BUTTONS: &[KeyCode] = &[KeyCode::KeyF, KeyCode::KeyJ];
const KA_BUTTONS: &[KeyCode] = &[KeyCode::KeyD, KeyCode::KeyK];

fn handle_key_inputs(mut commands: Commands, keyboard_input: Res<ButtonInput<KeyCode>>, mut query: Query<(Entity, &TimeUntilClick), With<CanHit>>){
    for (entity, time) in &mut query.iter().sort_unstable::<&TimeUntilClick>(){
        if is_key_group_hit(&keyboard_input, DON_BUTTONS){
            commands.entity(entity).despawn();
            break;
        }
    }

}

const UNITS_PER_SECOND: f32 = -500.;
const HIT_POSITION: Vec2 = Vec2::new(600., 100.);

fn update_note_transform(mut query: Query<(&mut Transform, &TimeUntilClick)>) {
    for (mut transform, time_until_click) in &mut query {
        let distance_until_hit = time_until_click.nano as f32/ MILLS_NANO_CONVER_FACTOR as f32  - LATEST_WINDOW_MILLS as f32;
        transform.translation.x =
            (distance_until_hit * UNITS_PER_SECOND) / 1000f32 + HIT_POSITION.x;
        transform.translation.y = HIT_POSITION.y;
    }
}

fn is_key_group_hit(keyboard_input: &Res<ButtonInput<KeyCode>>, keys: &[KeyCode]) -> bool {
    keys.iter()
        .filter(|k| keyboard_input.just_pressed(**k))
        .count()
        > 0
}

// These constants are defined in `Transform` units.
// Using the default 2D camera they correspond 1:1 with screen pixels.
// const PADDLE_SIZE: Vec2 = Vec2::new(120.0, 20.0);
// const GAP_BETWEEN_PADDLE_AND_FLOOR: f32 = 60.0;
// const PADDLE_SPEED: f32 = 500.0;
// // How close can the paddle get to the wall
// const PADDLE_PADDING: f32 = 10.0;

// We set the z-value of the ball to 1 so it renders on top in the case of overlapping sprites.
// const BALL_STARTING_POSITION: Vec3 = Vec3::new(0.0, -50.0, 1.0);
// const BALL_DIAMETER: f32 = 30.;
// const BALL_SPEED: f32 = 400.0;
// const INITIAL_BALL_DIRECTION: Vec2 = Vec2::new(0.5, -0.5);

// const WALL_THICKNESS: f32 = 10.0;
// // x coordinates
// const LEFT_WALL: f32 = -450.;
// const RIGHT_WALL: f32 = 450.;
// // y coordinates
// const BOTTOM_WALL: f32 = -300.;
// const TOP_WALL: f32 = 300.;

// const BRICK_SIZE: Vec2 = Vec2::new(100., 30.);
// // These values are exact
// const GAP_BETWEEN_PADDLE_AND_BRICKS: f32 = 270.0;
// const GAP_BETWEEN_BRICKS: f32 = 5.0;
// // These values are lower bounds, as the number of bricks is computed
// const GAP_BETWEEN_BRICKS_AND_CEILING: f32 = 20.0;
// const GAP_BETWEEN_BRICKS_AND_SIDES: f32 = 20.0;

const SCOREBOARD_FONT_SIZE: f32 = 33.0;
const SCOREBOARD_TEXT_PADDING: Val = Val::Px(5.0);

const BACKGROUND_COLOR: Color = Color::srgb(0.9, 0.9, 0.9);
// const PADDLE_COLOR: Color = Color::srgb(0.3, 0.3, 0.7);
const DON_COLOR: Color = Color::srgb(1.0, 0.5, 0.5);
const KA_COLOR: Color = Color::srgb(0.5, 0.5, 1.0);
const HIT_COLOR: Color = Color::srgb(0.1, 0.1, 0.1);
// const WALL_COLOR: Color = Color::srgb(0.8, 0.8, 0.8);
const TEXT_COLOR: Color = Color::srgb(0.5, 0.5, 1.0);
const SCORE_COLOR: Color = Color::srgb(1.0, 0.5, 0.5);

fn main() {
    App::new()
        .add_plugins(DefaultPlugins)
        .insert_resource(Score(0))
        .insert_resource(ClearColor(BACKGROUND_COLOR))
        .add_systems(Startup, setup)
        // Add our gameplay simulation systems to the fixed timestep schedule
        // which runs at 64 Hz by default
        .add_systems(Startup, spawn_notes)
        // .add_systems(
        //     FixedUpdate,
        //     ( (save_physics_position, move_paddle), apply_velocity, update_transform_for_collision_check, check_for_collisions)
        //         // `chain`ing systems together runs them in order
        //         .chain(),
        // )
        // .add_systems(Update, (update_scoreboard, update_visual_transform))
        .add_systems(
            Update,
            (update_timed, (update_note_transform, mark_pressable), handle_key_inputs, free_timed).chain(),
        )
        .add_observer(play_collision_sound)
        .run();
}

#[derive(Component)]
struct Paddle;

#[derive(Component)]
struct Ball;

#[derive(Component, Deref, DerefMut)]
struct Velocity(Vec2);

#[derive(Event)]
struct BallCollided;

#[derive(Component)]
struct Brick;

#[derive(Resource, Deref)]
struct CollisionSound(Handle<AudioSource>);

// Default must be implemented to define this as a required component for the Wall component below
#[derive(Component, Default)]
struct Collider;

// This is a collection of the components that define a "Wall" in our game
#[derive(Component)]
#[require(Sprite, Transform, Collider)]
struct Wall;

#[derive(Component, Default)]
struct PhysicsTransform(Vec2);

#[derive(Component)]
#[require(PhysicsTransform)]
struct PhysicsTransformInterpolate(Option<(Vec2, f64)>);
// This resource tracks the game's score
#[derive(Resource, Deref, DerefMut)]
struct Score(usize);

#[derive(Component)]
struct ScoreboardUi;

// Add the game's entities to our world
fn setup(
    mut commands: Commands,
    mut meshes: ResMut<Assets<Mesh>>,
    mut materials: ResMut<Assets<ColorMaterial>>,
    asset_server: Res<AssetServer>,
) {
    // Camera
    commands.spawn(Camera2d);

    commands.spawn((Sprite::from_color(HIT_COLOR, Vec2::new(100., 100.)), Transform::from_xyz(HIT_POSITION.x, HIT_POSITION.y, -1.)));

    // Sound
    let ball_collision_sound = asset_server.load("sounds/breakout_collision.ogg");
    commands.insert_resource(CollisionSound(ball_collision_sound));

    // Scoreboard
    commands.spawn((
        Text::new("Score: "),
        TextFont {
            font_size: SCOREBOARD_FONT_SIZE,
            ..default()
        },
        TextColor(TEXT_COLOR),
        ScoreboardUi,
        Node {
            position_type: PositionType::Absolute,
            top: SCOREBOARD_TEXT_PADDING,
            left: SCOREBOARD_TEXT_PADDING,
            ..default()
        },
        children![(
            TextSpan::default(),
            TextFont {
                font_size: SCOREBOARD_FONT_SIZE,
                ..default()
            },
            TextColor(SCORE_COLOR),
        )],
    ));
}

fn update_visual_transform(
    mut moving_query: Query<(
        &mut Transform,
        &PhysicsTransformInterpolate,
        &PhysicsTransform,
    )>,
    time: Res<Time>,
) {
    const FIXED_UPDATE_INTERVAL: f64 = 1. / 64.;
    for (mut transform, interpolate, current) in &mut moving_query {
        let current_pos = current.0;
        let (last_pos, last_t) = interpolate
            .0
            .unwrap_or((current_pos, time.elapsed_secs_f64()));

        let lerp_i = (time.elapsed_secs_f64() - last_t) / FIXED_UPDATE_INTERVAL;

        let interp_pos = last_pos.lerp(current_pos, lerp_i as f32);

        transform.translation.x = interp_pos.x;
        transform.translation.y = interp_pos.y;
    }
}

fn play_collision_sound(
    _collided: On<BallCollided>,
    mut commands: Commands,
    sound: Res<CollisionSound>,
) {
    commands.spawn((AudioPlayer(sound.clone()), PlaybackSettings::DESPAWN));
}
