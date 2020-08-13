use simulate::Simulation;
use simulate::Statistics;
use simulate::StatisticsTrackingSimulator;
use std::error::Error;

#[derive(Debug)]
struct AddOneSimulationStatistics(u32);
impl Statistics<u32> for AddOneSimulationStatistics {
  fn get_names() -> std::vec::Vec<std::string::String> {
    vec!["Value".into()]
  }

  fn get_value(&self, name: &str) -> f64 {
    match name {
      "Value" => self.0 as f64,
      _ => panic!("Bad"),
    }
  }

  fn derive(state: &u32) -> Self {
    Self(*state)
  }
}

struct AddOneSimulation;
impl Simulation for AddOneSimulation {
  type TState = u32;

  fn tick(prev_state: &u32) -> u32 {
    prev_state + 1
  }
}

fn main() -> Result<(), Box<dyn Error>> {
  let mut sim: StatisticsTrackingSimulator<AddOneSimulation, AddOneSimulationStatistics> =
    StatisticsTrackingSimulator::new(0);

  for _ in 0..10 {
    sim.tick();
  }

  println!("{:?}", sim.statistics().collect::<Vec<_>>());

  Ok(())
}
