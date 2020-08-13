pub trait Simulation {
  type TState;

  fn tick(prev_state: &Self::TState) -> Self::TState;
}

pub trait Statistics<T> {
  fn get_names() -> Vec<String>;
  fn get_value(&self, name: &str) -> f64;
  fn derive(state: &T) -> Self;
}

pub struct Simulator<TSimulation>
where
  TSimulation: Simulation,
{
  state: TSimulation::TState,
}

impl<TSimulation> Simulator<TSimulation>
where
  TSimulation: Simulation,
{
  pub fn new(init_state: TSimulation::TState) -> Self {
    Self { state: init_state }
  }

  pub fn tick(&mut self) {
    self.state = TSimulation::tick(self.state());
  }

  pub fn state(&self) -> &TSimulation::TState {
    &self.state
  }
}

pub struct StatisticsTrackingSimulator<TSimulation, TStatistics>
where
  TSimulation: Simulation,
  TStatistics: Statistics<TSimulation::TState>,
{
  statistics: Vec<TStatistics>,
  simulator: Simulator<TSimulation>,
}

impl<TSimulation, TStatistics> StatisticsTrackingSimulator<TSimulation, TStatistics>
where
  TSimulation: Simulation,
  TStatistics: Statistics<TSimulation::TState>,
{
  pub fn new(init_state: TSimulation::TState) -> Self {
    Self {
      statistics: vec![TStatistics::derive(&init_state)],
      simulator: Simulator::new(init_state),
    }
  }

  pub fn tick(&mut self) {
    self.simulator.tick();
    self.statistics.push(TStatistics::derive(self.state()));
  }

  pub fn state(&self) -> &TSimulation::TState {
    self.simulator.state()
  }

  pub fn statistics(&self) -> impl Iterator<Item = &TStatistics> {
    self.statistics.iter()
  }

  pub fn most_recent_statistics(&self) -> &TStatistics {
    self.statistics.last().unwrap()
  }
}
