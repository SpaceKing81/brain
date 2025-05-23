use macroquad::color::Color;
//Simulation
pub const GRID_SIZE: f32 = 40.0;
pub const GRAVITY: f32 = 0.01;
pub const GRAVITY_SUFRACE: f32 = 50.0;
pub const ELECTRIC_SUFRACE: f32 = 11.0;
pub const COULOMB: f32 = 1000.0;  
pub const SPRING:f32 = 1.0;
pub const SPRING_NORMAL:f32 = 20.0;
pub const TIME_STEP: f32 = 0.01;


// Neurons
pub const ONE_STANDARD_DEV_THRESHOLD:i32 = 30;
pub const ITERATION_MULTIPLIER: u32 = 5;

// Colors
pub const OUTPUT_COLOR:Color = Color::new(0.0, 0.5, 1.0, 1.0);
pub const AXON_NEG_COLOR:Color = Color::new(0.6, 0.2, 0.0, 0.5);
pub const AXON_POS_COLOR:Color = Color::new(0.1, 0.5, 0.3, 0.5);
pub const AXON_INPUT_COLOR:Color = Color::new(0.9, 0.3, 0.0, 0.5);