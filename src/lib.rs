#[cfg(feature = "ggez_app")]
pub mod ggez;

pub trait Simulation {
  type TState;

  fn tick(&mut self, state: &mut Self::TState);
}

pub trait Statistics<T> {
  fn get_title() -> String;
  fn get_unit() -> String;
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
  max_value: f64,
  min_value: f64,
  simulator: Simulator<TSimulation>,
  statistics: Vec<(usize, TStatistics)>,
  tick: usize,
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
    let stats = TStatistics::derive(&init_state);
    let values: Vec<_> = TStatistics::get_names()
      .iter()
      .map(|name| stats.get_value(name))
      .collect();
    Self {
      config,
      max_value: values.iter().cloned().fold(0.0, f64::max),
      min_value: values.iter().cloned().fold(0.0, f64::min),
      simulator: Simulator::new(simulation, init_state),
      statistics: vec![(0, stats)],
      tick: 0,
    }
  }

  pub fn tick(&mut self) {
    self.simulator.tick();
    self.tick += 1;
    if self.tick % self.config.step == 0 {
      let stats = TStatistics::derive(self.state());
      let values: Vec<_> = TStatistics::get_names()
        .iter()
        .map(|name| stats.get_value(name))
        .collect();
      self.max_value = f64::max(self.max_value, values.iter().cloned().fold(0.0, f64::max));
      self.min_value = f64::min(self.min_value, values.iter().cloned().fold(0.0, f64::min));
      self.statistics.push((self.tick, stats));
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
