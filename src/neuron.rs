// use std::collections::HashMap;
use macroquad::{
  color::*, math::Vec2, rand, shapes::*,
  window::{screen_width,screen_height},
};
use crate::consts::*;
const OUTPUT_COLOR:Color = Color::new(0.0, 0.5, 1.0, 1.0);



/// Single neuron
#[derive(Clone)]
pub struct Neuron {
  // id:u32, // name, basiclly
  pub position: Vec2, // Position on the screen
  base_threshold:u32,
  threshold:u32, // threshold to fire
  pub happyness:u32, // how happy it is with the firing frequency, 0 is happiest
  
  pub is_output: bool,

  pub inputs:Vec<i32>, // total inputs for this tick, post weight
  input_memory:Vec<i32>, // Memory of previous values, currently 5

  pub output_axions: Vec<u128>, 
  pub input_axions: Vec<u128>,

  tick_last_fired:u128, // the time it fired last
  pub delta_t:u32, // how long since last fire
  avg_t:u32, // average time since last firing
  
} 

// General
impl Neuron {
  /// Makes a new neuron
  // pub fn new(id:u32) -> Self {
  pub fn new(is_output:bool) -> Self {
    Neuron {
        // id,
        position:Vec2::new(rand::gen_range(0.0+20.0,screen_width()-20.0), rand::gen_range(0.0+10.0,screen_height()-10.0)),
        happyness:25,
        base_threshold:50,
        threshold:50,
        is_output,
        
        input_memory:vec![0,0,0,0,0],
        inputs:Vec::new(),

        output_axions: Vec::new(),
        input_axions: Vec::new(),
        
        tick_last_fired:0,
        delta_t:0,
        avg_t:0,
    }
  }
  /// Rolls a save check to see if it should die or gets another chance at life, and if so how many.
  /// Only relies on happyness value, but only really used if it doesnt have any outputs or inputs left.
  pub fn roll_save_check(&self, output:bool) -> Option<i32> {
    if output {return Some(rand::gen_range(10,20))}
    let roll = rand::gen_range(0,MAX_HAPPY_VALUE/2);
    if roll + self.happyness < MAX_HAPPY_VALUE/5 {
      return Some(rand::gen_range(10,20))
    }
    None
  }
}
// Update
impl Neuron {
  /// Honestly useless, just sets the tick to 0, returns time since last fired
  pub fn fired(&mut self) -> u32 {
    let delta_t = self.delta_t;
    self.delta_t = 0;
    delta_t
  }
  
  /// Housekeeping stuff, memory management, time updating, basic universal update.
  /// Updates everything that needs to be refreshed whenever it becomes an active neuron.
  pub fn update(&mut self, time:u128) {
    // Clock update, updates action as needed
    let old_time = self.delta_t;
    self.tick(time);

    // Update threshold values
    self.update_threshold();
    self.math();
    // Update the memory as fit
    if old_time + 5 <= self.delta_t {
      // Voids memory and replaces it with the current update scheme
      self.forget(None);
    } else {
      // Updates memory as accurate to the times
      self.forget(Some((self.delta_t-old_time)as usize));
    }
  }
  
  fn tick(&mut self, time:u128) {
    self.delta_t = (time - self.tick_last_fired) as u32;
  } 
  fn math(&mut self) {
    let t = self.delta_t as i32;
    let w = self.avg_t as i32;
    let delta = (w - t).abs();
    let weight = (delta/ONE_STANDARD_DEV_THRESHOLD).abs() as u32;
    
    // alters the reliablity by the distence from average
    self.happyness += weight;
    if weight == 0 {self.happyness = self.happyness.saturating_sub(5);}
    if weight <= 2 {self.happyness = self.happyness.saturating_sub(2)}
    self.avg_t = (self.avg_t + self.avg_t + self.delta_t) / 3;
  }
  
}

// Graphics
impl Neuron {
  /// Draws the Neuron where ever it is, gives it a color based on its firing status
  pub fn draw(&self) {
    if self.is_output { draw_circle(self.position.x, self.position.y, 10.0, OUTPUT_COLOR); return;}
    let color = if self.delta_t == 0 {RED} else if self.delta_t < 5 {YELLOW} else {GRAY};
    draw_circle(self.position.x, self.position.y, 10.0, color);
  }

}
// Output stuff
impl Neuron {
  /// Checks if the neuron wants to fire 
  pub fn ready_to_fire(&self) -> bool {
    if self.delta_t <= 5 {return false}
    let potential:i32 = self.input_memory.iter().sum();
    potential.abs() as u32 >= self.threshold
  } 
  /// Checks if the neuron should be killed
  pub fn check_to_kill(&self) -> bool {
    if self.is_output {return false}
    if self.happyness >= MAX_HAPPY_VALUE {return true}
    if self.delta_t > INACTIVITY_DEATH_TIME {return true}
    false
  }
}
// Input
impl Neuron {
  /// Either specify how many lost seconds, or complete replacement if none for memory
  fn forget(&mut self, ticks:Option<usize>) {
    let sum:i32 = self.inputs.iter().sum();
    if ticks == Some(0) {return;}
    match ticks {
      None => {
        // get the current sum value
        let mut input_memory = vec![sum];
        input_memory.append(&mut vec![0;MEMORY_SIZE-1]);
        self.inputs.clear();
      },
      Some(u) => {
        let u = if u<MEMORY_SIZE {u}else{MEMORY_SIZE};
        for _i in 0..u {self.input_memory.pop();}
        let sum:i32 = self.inputs.iter().sum();
        let mut new = vec![sum];
        new.append(&mut vec![0;u-1]);
        new.append(&mut self.input_memory);
        self.input_memory = new;
      },
    }
  }

}

// Mutate Thresholds
impl Neuron {
  /// Updates the threshold based on frequency and happyness, along with factoring in a cooldown period
  pub fn update_threshold(&mut self) {
    if self.ready_to_fire() {
      let potential:i32 = self.input_memory.iter().sum();
      let extreme_fire:bool = potential.abs() > 100;
      let u = (self.delta_t as i32) - (self.avg_t as i32);
      let w = u / ONE_STANDARD_DEV_THRESHOLD;
      if w.abs() <= 1 {
        self.happyness = self.happyness.saturating_sub(4);

      } else if w.abs() > 4 {
        if (w.is_negative() && self.base_threshold >= 70) || extreme_fire {
            self.happyness += w.abs() as u32;
        } else if w.is_negative() || extreme_fire {
            self.raise_base_threshold();
        }
        if w.is_positive() && self.base_threshold <= 30 {
          self.happyness += w.abs() as u32;
        } else if w.is_positive() {
            self.drop_base_threshold();
        }
      }
    } else {
      self.post_fire_threshold();
    }
  }

  // alters actual threshold when its right after firing
  fn post_fire_threshold(&mut self) {
    if self.delta_t >= 15 { self.set_threshold_to_normal(); }
    if self.delta_t >= 5 { self.set_threshold_to_increment(); }
    if self.delta_t < 5 { self.set_threshold_to_recovery(); }
  } 

  // ease of reading fns
  fn set_threshold_to_normal(&mut self) {
    self.threshold = self.base_threshold;
  }
  fn set_threshold_to_increment(&mut self) {
    self.threshold = self.threshold.saturating_sub(3 * self.delta_t - 15);
  }
  fn set_threshold_to_recovery(&mut self) {
    self.threshold = self.base_threshold + 30;
  }
  
  // Base Threshold shifts
  fn drop_base_threshold(&mut self) {
    let u = (self.delta_t as i32) - (self.avg_t as i32);
    let w = u / ONE_STANDARD_DEV_THRESHOLD;
    let w = w.abs() as u32;
    self.base_threshold = self.base_threshold.saturating_sub(w);
    if self.base_threshold < MIN_THRESHOLD { self.base_threshold = MIN_THRESHOLD; }
    self.post_fire_threshold();
  }
  fn raise_base_threshold(&mut self) {
    let u = (self.delta_t as i32) - (self.avg_t as i32);
    let w = u / ONE_STANDARD_DEV_THRESHOLD;
    let w = w.abs() as u32;
    self.base_threshold += w;
    if self.base_threshold > MAX_THRESHOLD { self.base_threshold = MAX_THRESHOLD; }
    self.post_fire_threshold();
  }
  
}

