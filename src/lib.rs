pub mod stats;

#[cfg(feature = "ggez_app")]
pub mod ggez;

pub trait Simulation {
  type TState;

  fn tick(&mut self, state: &mut Self::TState);
}

pub struct Simulator<TSimulation>
where
  TSimulation: Simulation,
{
  simulation: TSimulation,
  state: TSimulation::TState,
}

impl<TSimulation> Simulator<TSimulation>
where
  TSimulation: Simulation,
{
  pub fn new(simulation: TSimulation, init_state: TSimulation::TState) -> Self {
    Self {
      simulation,
      state: init_state,
    }
  }

  pub fn tick(&mut self) {
    self.simulation.tick(&mut self.state);
  }

  pub fn state(&self) -> &TSimulation::TState {
    &self.state
  }
}
