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
const HIT_POSITION: Vec2 = Vec2::new(500., 100.);

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
        // .add_observer(play_collision_sound)
        .run();
}



#[derive(Resource, Deref)]
struct CollisionSound(Handle<AudioSource>);

// This resource tracks the game's score
#[derive(Resource, Deref, DerefMut)]
struct Score(usize);

#[derive(Component)]
struct ScoreboardUi;

// Add the game's entities to our world
fn setup(
    mut commands: Commands,
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

// fn play_collision_sound(
//     _collided: On<BallCollided>,
//     mut commands: Commands,
//     sound: Res<CollisionSound>,
// ) {
//     commands.spawn((AudioPlayer(sound.clone()), PlaybackSettings::DESPAWN));
// }
