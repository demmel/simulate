#[cfg(feature = "ggez_app")]
pub mod ggez;

pub trait Simulation {
  type TState;

  fn tick(&mut self, state: &mut Self::TState);
}

pub trait Statistics<T> {
  fn get_title() -> String;
  fn get_unit() -> String;
  fn get_unit_min() -> f64;
  fn get_unit_max() -> f64;
  fn get_tick_unit() -> String;
  fn map_tick_unit(tick: usize) -> f64;
  fn get_names() -> Vec<String>;
  fn get_value(&self, name: &str) -> f64;
  fn derive(state: &T) -> Self;
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

pub struct StatisticsTrackingSimulatorConfig {
  step: usize,
}

impl Default for StatisticsTrackingSimulatorConfig {
  fn default() -> Self {
    StatisticsTrackingSimulatorConfig { step: 1 }
  }
}

impl StatisticsTrackingSimulatorConfig {
  pub fn step(mut self, step: usize) -> Self {
    self.step = step;
    self
  }
}

pub struct StatisticsTrackingSimulator<TSimulation, TStatistics>
where
  TSimulation: Simulation,
  TStatistics: Statistics<TSimulation::TState>,
{
  config: StatisticsTrackingSimulatorConfig,
  tick: usize,
  statistics: Vec<(usize, TStatistics)>,
  simulator: Simulator<TSimulation>,
}

impl<TSimulation, TStatistics> StatisticsTrackingSimulator<TSimulation, TStatistics>
where
  TSimulation: Simulation,
  TStatistics: Statistics<TSimulation::TState>,
{
  pub fn new(simulation: TSimulation, init_state: TSimulation::TState) -> Self {
    Self::with_config(
      simulation,
      init_state,
      StatisticsTrackingSimulatorConfig::default(),
    )
  }

  pub fn with_config(
    simulation: TSimulation,
    init_state: TSimulation::TState,
    config: StatisticsTrackingSimulatorConfig,
  ) -> Self {
    Self {
      config,
      tick: 0,
      statistics: vec![(0, TStatistics::derive(&init_state))],
      simulator: Simulator::new(simulation, init_state),
    }
  }

  pub fn tick(&mut self) {
    self.simulator.tick();
    self.tick += 1;
    if self.tick % self.config.step == 0 {
      self
        .statistics
        .push((self.tick, TStatistics::derive(self.state())));
    }
  }

  pub fn state(&self) -> &TSimulation::TState {
    self.simulator.state()
  }

  pub fn statistics(&self) -> impl Iterator<Item = &(usize, TStatistics)> {
    self.statistics.iter()
  }

  pub fn most_recent_statistics(&self) -> &(usize, TStatistics) {
    self.statistics.last().unwrap()
  }
}
