// Blobby Volley 3 - Physical and Game Constants
// Faithfully matching the authentic Blobby Volley 2 physics model

/// Virtual play area dimensions
pub const VIRTUAL_WIDTH: f32 = 800.0;
pub const VIRTUAL_HEIGHT: f32 = 600.0;

// Border boundaries
pub const LEFT_PLANE: f32 = 0.0;
pub const RIGHT_PLANE: f32 = 800.0;

// Ground plane
pub const GROUND_PLANE_HEIGHT_MAX: f32 = 500.0;

// Blobby dimensions
pub const BLOBBY_HEIGHT: f32 = 89.0;
pub const GROUND_PLANE_HEIGHT: f32 = GROUND_PLANE_HEIGHT_MAX - BLOBBY_HEIGHT / 2.0; // 455.5
pub const BLOBBY_UPPER_SPHERE: f32 = 19.0;
pub const BLOBBY_UPPER_RADIUS: f32 = 25.0;
pub const BLOBBY_LOWER_SPHERE: f32 = 13.0;
pub const BLOBBY_LOWER_RADIUS: f32 = 33.0;

// Jump and gravity mechanics
pub const BLOBBY_MAX_JUMP_HEIGHT: f32 = GROUND_PLANE_HEIGHT - 206.375; // 249.125
pub const BLOBBY_JUMP_ACCELERATION: f32 = -15.1;
pub const GRAVITATION: f32 = (BLOBBY_JUMP_ACCELERATION * BLOBBY_JUMP_ACCELERATION) / BLOBBY_MAX_JUMP_HEIGHT; // ~0.91522
pub const BLOBBY_JUMP_BUFFER: f32 = GRAVITATION / 2.0; // ~0.45761
pub const BLOBBY_SPEED: f32 = 4.5;

// Ball dimensions and physics
pub const BALL_RADIUS: f32 = 31.5;
pub const BALL_GRAVITATION: f32 = 0.287;
pub const BALL_COLLISION_VELOCITY: f32 = 13.1225; // (0.75 * RIGHT_PLANE * BALL_GRAVITATION).sqrt()
pub const STANDARD_BALL_HEIGHT: f32 = 269.0 + BALL_RADIUS; // 300.5
pub const STANDARD_BALL_ANGULAR_VELOCITY: f32 = 0.1;

// Volley ball net
pub const NET_POSITION_X: f32 = RIGHT_PLANE / 2.0; // 400.0
#[allow(dead_code)]
pub const NET_POSITION_Y: f32 = 438.0;
pub const NET_RADIUS: f32 = 7.0;
pub const NET_SPHERE_POSITION: f32 = 284.0; // Top round cap y

// Hit tolerance frames: 25 ticks (~333ms) prevents multi-contact phantom touches on a single jump/header
pub const SQUISH_TOLERANCE: i32 = 25;

// Simulation rate: 75 updates per second
pub const PHYSICS_TICK_RATE: f32 = 75.0;
pub const PHYSICS_DT: f32 = 1.0 / PHYSICS_TICK_RATE;
