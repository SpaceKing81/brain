use macroquad::prelude::*;
// use std::collections::{HashMap, HashSet};
mod neuron;
mod axion;
mod brain;
mod input;
mod output;

use neuron::Neuron;
use axion::Axion;
use brain::Brain;
use input::Input;
use output::Output;
/// Calculate Modulus operations
fn modulo<T>(a: T, b: T) -> T where T: std::ops::Rem<Output = T> + std::ops::Add<Output = T> + Copy, {((a % b) + b) % b}
fn window_conf() -> Conf {
    Conf {
        window_title: "Brain Simulation".to_owned(),
        fullscreen: true,
        window_resizable: true,
        ..Default::default()
    }
}

const STARTING_NEURONS:u32 = 200;
const STARTING_INPUTS:u32 = 10;
const IDEAL_TPS:f64 = 60.0;



#[macroquad::main(window_conf)]
async fn main() {
    
    // Initialize the brain
    println!("Starting simulation...");
    let mut brain = Brain::new();
    brain.spin_up_new(STARTING_NEURONS,STARTING_INPUTS);
    let center = Vec2::new(screen_width()/2.0, screen_height()/2.0);
    println!("Brain initialized. Entering continuous operations...");
    let mut ticks = 0.0;
    // Main loop
    loop {
        // Handle Ending
        if is_key_down(KeyCode::Escape) {
            println!("Terminating Brain...");
            break;
        }
        // Update the brain
        loop {
            // let fire = (modulo(get_time(),5.0)) as i32 == 0;
            let fire = if !is_key_down(KeyCode::S) {(modulo(get_time(),5.0)) as i32 == 0 } else { false };
            // let fire = get_time() >= 10.0 && get_time() <= 11.0;
            
            println!("new tick: {}", fire);
            brain.tick(fire);
            let time = if get_time() == 0.0 {0.02} else {get_time()};
            if ticks/time < IDEAL_TPS || is_key_down(KeyCode::Escape){ break; }
        }
        // Drawing a frame
        { 

        // Clear the screen
        clear_background(BLACK);

        // Update and draw neurons and axons
        brain.general_update(center);
        // Draw FPS and other info
        draw_text(
            &format!("TPS: {}, FPS: {}", (ticks/get_time()).round(), get_fps()),
            screen_width() - 200.,
            20.,
            20.,
            WHITE,
        );
        ticks += 1.0;
        if brain.neurons.len() == 0 || brain.axions.len() == 0 { break; };
        }
        // Render the frame
        next_frame().await;
    }
}


/*
TODO:
- Fix the spin up so that it is actually working and somewhat efficnet
- Expand the input to be better
- Need to make a special tick that goes through and updates all the neurons
    - compile all the logic so that it can iterate through everything once, and not have to do multiple for loops
    for things like drawing, and then updating, and then general updating, blah blah blah. Consolidate everything so that
    it can be combined into a single fn under a single for loop for every input, then every axion, then neuron, then output.

- ITS NOT CRASHING FOR ONCE!!!!
-Because i am me, i imedietly tried to break it to see what happens:
    - Max neuron number before frame preformence suffers: 200 neurons at 60 TPS
    - Oddly, the less neurons there are, the more jumpy the whole thing becomes
    - Good TPS is around 60


NOTES:
- threshold value can fluctuate between 30 and 70 in a neuron. The node records the natural threshold value,
and this can then change based on average firing rate. Once the node fires, the threshold value is updated based
on the average rate, and then artificialy boosted by the reload function, which is constant. 

UNKNOWN GOAL: Each node operates individually, running basic calculations continuously like 
tick and memory increments, and modulating thresholds post firing and slow alteration of axon weights with disuse.
Firing functions kick in and the whole system gets activated from semi-dormant state when surpassing threshold,
pushing an update to all the connected nodes who then possibly update as well.

*/