use std::collections::{HashMap, HashSet};
// use rayon::prelude::*;
use macroquad::{
  // color::*, 
  math::Vec2, rand, shapes::*
  // window::{screen_width,screen_height},
};
use std::u128;

use crate::{
  //
  Axion, Input, Neuron, Output, grid::{grid::*, update_threads::*},consts::*,
  //
};


pub struct Brain {
  clock:u128,

  pub neurons: HashMap<u32, Neuron>,
  pub axions: HashMap<u128,Axion>,
  pub inputs:HashMap<u32, Input>,
  pub outputs:HashMap<u32, Output>,

  num_of_neurons: u32,
  num_of_axions: u128,
  num_of_inputs: u32,
  num_of_outputs: u32,


  active_neurons:HashSet<u32>,
}

impl Brain {
  pub fn new() -> Brain {
    Brain {
      clock:0,

      neurons: HashMap::new(),
      axions: HashMap::new(),
      inputs:HashMap::new(),
      outputs:HashMap::new(),

      num_of_neurons:0,
      num_of_axions:0,
      num_of_inputs: 0,
      num_of_outputs: 0,


      active_neurons:HashSet::new(),
    }
  }
  pub fn spin_up_new(&mut self, num_neurons: u32, num_input: u32) {
    // Step 1: Create neurons
    for _ in 0..(num_neurons + 10) {
        self.add_neuron();
    }

    // Cache neuron IDs into a Vec for efficient random access
    let neuron_ids: Vec<u32> = self.neurons.keys().copied().collect();
    let len = neuron_ids.len();

    // Step 2: Connect neurons with axons
    for _ in 0..(num_neurons * 2) {
        let source_id = neuron_ids[rand::gen_range(0, len)];
        let sink_id = neuron_ids[rand::gen_range(0, len)];
        if source_id != sink_id {
            self.add_axion(source_id, sink_id);
            self.num_of_axions += 1;
        }
    }

    // Identify and mark neurons without outputs
    for id in &neuron_ids[..num_neurons as usize] {
        if let Some(neuron) = self.neurons.get(id) {
            if neuron.output_axions.is_empty() {
                self.no_more_outputs(*id);
            }
        }
    }

    // Step 3: Set up input neurons
    self.num_of_inputs = num_input;
    for id in 0..num_input {
        let mut input = Input::new(id);

        let connect_count = rand::gen_range(1, std::cmp::max(self.num_of_neurons, 2));
        let mut seen = std::collections::HashSet::with_capacity(connect_count as usize);

        while seen.len() < connect_count as usize {
            let i = neuron_ids[rand::gen_range(0, len)];
            if seen.insert(i) && self.neurons.contains_key(&i) {
                input.output_neurons.push(i);
            }
        }

        self.inputs.insert(id, input);
    }

    // TEMP--TB DELEATED
    self.temp_for_output();
}
  pub fn tick(&mut self, input:bool) {
    // one tick passes
    self.clock += 1;
    // if an input is triggered
    if input {
      for id in 0..self.num_of_inputs {
        self.active_neurons.extend(self.inputs[&id].output_neurons.clone());
      }
    }

    let active_neurons_to_iter: Vec<u32> = self.active_neurons.drain().collect();
    let active_neurons: HashSet<u32> = active_neurons_to_iter.iter().copied().collect();
    
    let mut axions_to_remove = Vec::new();
    let mut neurons_to_remove= Vec::new();

    println!("{} neruons to fire this tick", active_neurons.len());
    for neuron_id in active_neurons_to_iter {
      if let Some(neuron) = self.neurons.get_mut(&neuron_id) {
        // update the input neuron happyness
        let input_axions = neuron.input_axions.clone();
        let happyness = neuron.happyness;
        for axion_id in input_axions {
          if let Some(axion) = self.axions.get_mut(&axion_id) {
            axion.update_happyness(happyness);
          }
        }
        // update the neurons
        neuron.update(self.clock);
        // Check if it should die
        if neuron.check_to_kill() {neurons_to_remove.push(neuron_id) }
        // check if it should fire
        if neuron.ready_to_fire() {
          // get the outputs
          let delta_t = neuron.delta_t;
          let output_axions = neuron.output_axions.clone();
          neuron.fired();

          for axion_id in output_axions {
            if let Some(axion) = self.axions.get_mut(&axion_id) {
              let (input_id, strength) = axion.fire(delta_t);
              if strength != 0 {
                // update all the input neuron strenght memories
                if let Some(input_neuron) = self.neurons.get_mut(&input_id) {
                  input_neuron.inputs.push(strength);
                // add them to the next active neuron cycle if its not a repeat
                  if !active_neurons.contains(&input_id) {
                    self.active_neurons.insert(input_id);
                  }
                
                }
              } else {axions_to_remove.push(axion_id);}
            }
          }
        }    
      }
    }
    // remove all inactive neurons
    for axion_id in axions_to_remove {self.remove_axion(axion_id);}
    for neuron_id in neurons_to_remove {self.no_more_outputs(neuron_id);}
    println!("Num of total neurons: {}", self.neurons.len());

  }
  
  pub fn general_update(&mut self, center: Vec2) {
    let mut neurons_to_remove: Vec<u32> = Vec::new();
    let mut axions_to_remove = Vec::new();
  
    // Update inputs and axions
    for input in self.inputs.values_mut() {
        input.update();
        input.draw();
    }

    for (&id, axion) in self.axions.iter() {
        if axion.strength == 0 {
          axions_to_remove.push(id);
        }
        self.draw_axion(axion);
    }

    // Step 1: build grid
    let grid = GridCell::build_spatial_grid(&self.neurons);

    // Step 2: do parallel update
    let (
      neuron_updates, 
      axion_updates
      ) = parallel_neuron_step(
        &self.neurons,
        &self.axions,
        &grid,
        center,
        |id, c| self.center_force(id, c),
        |a, b| self.spring_force(a, b),
    );

    // Step 3: apply changes serially
    for n_up in neuron_updates {
        if let Some(neuron) = self.neurons.get_mut(&n_up.id) {
          if neuron.check_to_kill() {
            neurons_to_remove.push(n_up.id);
            return;
          }
          neuron.position = n_up.new_position;
          neuron.update(self.clock);
        }
    }

    for a_up in axion_updates {
        if let Some(axion) = self.axions.get_mut(&a_up.id) {
            axion.update_happyness(a_up.new_happyness);
        }
    }

    // Step 4: draw neurons and outputs
    for neuron in self.neurons.values() {
        neuron.draw();
    }

    for output in self.outputs.values_mut() {
        output.update();
        output.draw();
    }
  }
  
  
  // Can be found and removed at the end of spin up new
  fn temp_for_output(&self) {
    if &self.num_of_outputs == &5 {
      return
    }
  }

}

/// Mechanics
impl Brain {
  fn spring_force(&self, id1:u32, id2:u32) -> Option<Vec2> {
    if id1 != id2 {return None}
    let pos1 = self.neurons[&id1].position;
    let pos2 = self.neurons[&id2].position;
    let distance_s = pos1.distance(pos2);

    if distance_s > SPRING_NORMAL { 
      let direction_s = (pos1 - pos2) / distance_s;
      let spring = SPRING * distance_s;
      return Some(spring * direction_s * TIME_STEP);
    }
    None
  }
  fn center_force(&self, id1:u32, center:Vec2) -> Option<Vec2> {
    let pos1 = self.neurons[&id1].position;
    let distance_g = pos1.distance(center);
    if distance_g > GRAVITY_SUFRACE { 
      let direction_g = (pos1 - center) / distance_g;
      let gravity = GRAVITY * distance_g;
      return Some(gravity * direction_g * TIME_STEP)
    }
  None
  }
  fn electric_force(&self, id1:u32, id2:u32) -> Option<Vec2> {
    if id1 == id2 {return None}
    let pos1 = self.neurons[&id1].position;
    let pos2 = self.neurons[&id2].position;
    // Like-Charge Repulsion
    let distance_e = pos1.distance(pos2);
    if distance_e > ELECTRIC_SUFRACE { // Prevent division by zero
      let direction_e = (pos1 - pos2) / distance_e;
      let electric = COULOMB / (distance_e * distance_e);
      return Some(electric * direction_e * TIME_STEP);
    }
    None
  }
}

/// Graphics
impl Brain {
  pub fn update_layout(&mut self, center:Vec2) {
    let neuron_ids: Vec<u32> = self.neurons.keys().cloned().collect();
    let mut new_positions: HashMap<u32, Vec2> = HashMap::new();

    // Spring Attraction
    for (_id,axion) in &self.axions {
      let source = axion.id_source;
      let sink = axion.id_sink;
      let pos1 = self.neurons.get(&source).unwrap().position.clone();
      let pos2 = self.neurons.get(&sink).unwrap().position.clone();
      let distance_s = pos1.distance(pos2);
      if distance_s > 0.0 { // Prevent division by zero
        let direction_s = (pos1 - pos2) / distance_s;
        let spring = SPRING * distance_s;
        let delta = spring * direction_s * TIME_STEP;
        new_positions.entry(source).and_modify(|p| *p -= delta).or_insert(-delta);
        new_positions.entry(sink).and_modify(|p| *p += delta).or_insert(delta);
      }
    }
    for &id1 in &neuron_ids {
      let mut delta = Vec2::ZERO;
      let pos1 = self.neurons[&id1].position;
      // Gravity Attraction
      let distance_g = pos1.distance(center);
      if distance_g > GRAVITY_SUFRACE { // Prevent division by zero
        let direction_g = (pos1 - center) / distance_g;
        let gravity = GRAVITY * distance_g;
        delta -= gravity * direction_g * TIME_STEP;
      }
      for &id2 in &neuron_ids {
        if id1 != id2 {
          let pos2 = self.neurons[&id2].position;
          // Like-Charge Repulsion
          let distance_e = pos1.distance(pos2);
          if distance_e > ELECTRIC_SUFRACE { // Prevent division by zero
            let direction_e = (pos1 - pos2) / distance_e;
            let electric = COULOMB / (distance_e * distance_e);
            delta += electric * direction_e * TIME_STEP;
          }
        }
      }
      if let Some(offset_spring) = new_positions.get(&id1) {
        self.neurons.entry(id1).and_modify(|p| p.position += offset_spring.clone() + delta);
      } else {
        self.neurons.entry(id1).and_modify(|p| p.position += delta);
      }
    }
  }
  pub fn draw(&self) {

    // Draw axons first (so they are behind neurons)
    for axion in self.axions.values() {
      self.draw_axion(axion);
    }
    // Draw neurons
    for neuron in self.neurons.values() {
      neuron.draw();
    }
    for input in self.inputs.values() {
      input.draw();
      // Crimson
    }
    for output in self.outputs.values() {
      output.draw();
    }
}
  
  fn draw_axion(&self, axion:&Axion) {
    let (source_id, sink_id, color) = axion.get_to_draw();
      if let (Some(source), Some(sink)) = (
        self.neurons.get(&source_id),
        self.neurons.get(&sink_id),
      ) {
        draw_line(
          source.position.x,
          source.position.y,
          sink.position.x,
          sink.position.y,
          2.0,
          color,
        );
      }
  }
}





impl Brain {
  pub fn no_more_outputs(&mut self, neuron_id: u32) {
    if let Some(neuron) = self.neurons.get(&neuron_id) {
      let roll = rand::gen_range(0,70);

      if roll + neuron.happyness < 50 {
        // Create new connections
        for _ in 0..rand::gen_range(5,10) {
          let sink_id = *self.neurons.keys().nth(rand::gen_range(0,self.neurons.len())).unwrap();
          if sink_id != neuron_id {
            self.add_axion(neuron_id, sink_id);
          }
        }
      } else {
        // Commit suicide
        self.remove_neuron(neuron_id);
      }
    }
  }
  
  fn combine_axions(input_axions: &Vec<u128>, output_axions: &Vec<u128>) -> HashSet<u128> {
    input_axions.iter().copied().chain(output_axions.iter().copied()).collect()
}


  fn add_neuron(&mut self) -> u32 {
    self.num_of_neurons +=1;
    let id = self.neurons.keys().max().unwrap_or(&0) + 1; // Generate a unique ID
    self.neurons.insert(id, Neuron::new());
    id
  }
  fn add_axion(&mut self, source_id: u32, sink_id: u32) -> u128 {
    self.num_of_axions +=1;
    let id = self.axions.keys().max().unwrap_or(&0) + 1; // Generate a unique ID
    let axion = Axion::new(source_id, sink_id, id);
    self.axions.insert(id, axion);

    // Update neuron connections
    if let Some(source_neuron) = self.neurons.get_mut(&source_id) {
      source_neuron.output_axions.push(id);
    }
    if let Some(sink_neuron) = self.neurons.get_mut(&sink_id) {
      sink_neuron.input_axions.push(id);
    }

    id
  }

  fn remove_neuron(&mut self, neuron_id: u32) {
      if let Some(neuron) = self.neurons.remove(&neuron_id) {
          // Remove all input axons
          self.num_of_neurons -= 1;
          for axion_id in neuron.input_axions {
              self.remove_axion(axion_id);
          }
          // Remove all output axons
          for axion_id in neuron.output_axions {
              self.remove_axion(axion_id);
          }
      }
  }
  fn remove_axion(&mut self, axion_id: u128) {
    if let Some(axion) = self.axions.remove(&axion_id) {
      // Remove axon from source neuron's output list
      if let Some(source_neuron) = self.neurons.get_mut(&axion.id_source) {
        source_neuron.output_axions.retain(|&id| id != axion_id);
      }
      // Remove axon from sink neuron's input list
      if let Some(sink_neuron) = self.neurons.get_mut(&axion.id_sink) {
        sink_neuron.input_axions.retain(|&id| id != axion_id);
      }
    }
  }

}


// need a list for all the neurons
// need a list for all the axions

// need a count for all the neurons
// need a count for all the axions

// need a list of input-nodes
// need a list of output-nodes
// need special connection for the input nodes -> neurons
// need special connection for the neurons -> output nodes

// need a master clock
// need a list of all activated neurons