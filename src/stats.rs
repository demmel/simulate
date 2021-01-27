use crate::Simulation;
use crate::Simulator;
use std::marker::PhantomData;

pub trait Statistics<T>: Sized {
  type TStatID: std::fmt::Display + Clone;

  fn get_tick_unit() -> String;
  fn map_tick_unit(tick: usize) -> f64;
  fn get_groups() -> Vec<StatisticsGroup<T, Self>>;
  fn get_value(&self, name: Self::TStatID) -> f64;
  fn derive(state: &T) -> Self;
}

pub struct StatisticsGroup<TState, TStatistics: Statistics<TState>> {
  pub title: String,
  pub unit: String,
  pub names: Vec<TStatistics::TStatID>,
  _state: PhantomData<TState>,
  _statistics: PhantomData<TStatistics>,
}

impl<TState, TStatistics: Statistics<TState>>
  StatisticsGroup<TState, TStatistics>
{
  pub fn new(
    title: &str,
    unit: &str,
    names: Vec<TStatistics::TStatID>,
  ) -> Self {
    Self {
      title: title.into(),
      unit: unit.into(),
      names,
      _state: PhantomData,
      _statistics: PhantomData,
    }
  }

  fn get_max_value(&self, stats: &TStatistics) -> f64 {
    self
      .names
      .iter()
      .map(|name| stats.get_value(name.clone()))
      .fold(0.0, f64::max)
  }

  fn get_min_value(&self, stats: &TStatistics) -> f64 {
    self
      .names
      .iter()
      .map(|name| stats.get_value(name.clone()))
      .fold(0.0, f64::min)
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

pub struct SimStats<TState, TStatistics: Statistics<TState>> {
  pub max_values: Vec<f64>,
  pub min_values: Vec<f64>,
  pub statistics: Vec<(usize, TStatistics)>,
  _state: PhantomData<TState>,
}

impl<TState, TStatistics: Statistics<TState>> SimStats<TState, TStatistics> {
  fn new(init_state: &TState) -> Self {
    let stats = TStatistics::derive(init_state);
    Self {
      max_values: TStatistics::get_groups()
        .iter()
        .map(|group| group.get_max_value(&stats))
        .collect(),
      min_values: TStatistics::get_groups()
        .iter()
        .map(|group| group.get_min_value(&stats))
        .collect(),
      statistics: vec![(0, stats)],
      _state: PhantomData,
    }
  }

  fn record(&mut self, tick: usize, state: &TState) {
    let stats = TStatistics::derive(state);
    self.max_values = self
      .max_values
      .iter()
      .zip(
        TStatistics::get_groups()
          .iter()
          .map(|group| group.get_max_value(&stats)),
      )
      .map(|(a, b)| f64::max(*a, b))
      .collect();
    self.min_values = self
      .min_values
      .iter()
      .zip(
        TStatistics::get_groups()
          .iter()
          .map(|group| group.get_min_value(&stats)),
      )
      .map(|(a, b)| f64::min(*a, b))
      .collect();
    self.statistics.push((tick, stats));
  }
}

pub struct StatisticsTrackingSimulator<TSimulation, TStatistics>
where
  TSimulation: Simulation,
  TStatistics: Statistics<TSimulation::TState>,
{
  config: StatisticsTrackingSimulatorConfig,
  simulator: Simulator<TSimulation>,
  pub stats: SimStats<TSimulation::TState, TStatistics>,
  tick: usize,
}

impl<TSimulation, TStatistics>
  StatisticsTrackingSimulator<TSimulation, TStatistics>
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
      stats: SimStats::new(&init_state),
      simulator: Simulator::new(simulation, init_state),
      tick: 0,
    }
  }

  pub fn tick(&mut self) {
    self.simulator.tick();
    self.tick += 1;
    if self.tick % self.config.step == 0 {
      self.stats.record(self.tick, self.simulator.state());
    }
  }

  pub fn state(&self) -> &TSimulation::TState {
    self.simulator.state()
  }

  pub fn statistics(&self) -> impl Iterator<Item = &(usize, TStatistics)> {
    self.stats.statistics.iter()
  }

  pub fn most_recent_statistics(&self) -> &(usize, TStatistics) {
    self.stats.statistics.last().unwrap()
  }
}

pub struct StatisticsDisplay<'a, S, T>
where
  S: Statistics<T>,
{
  stats: &'a S,
  _t: PhantomData<T>,
}

impl<'a, S, T> StatisticsDisplay<'a, S, T>
where
  S: Statistics<T>,
{
  pub fn new(stats: &'a S) -> Self {
    Self {
      stats,
      _t: PhantomData,
    }
  }
}

impl<'a, S, T> std::fmt::Display for StatisticsDisplay<'a, S, T>
where
  S: Statistics<T>,
{
  fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
    for group in S::get_groups() {
      writeln!(f, "{}", group.title)?;
      let name_strings: Vec<_> =
        group.names.iter().map(|name| format!("{}", name)).collect();
      let longest = name_strings.iter().map(|s| s.len()).max();

      for (i, name) in group.names.iter().enumerate() {
        writeln!(
          f,
          "  {:width$} : {} {}",
          name_strings[i],
          self.stats.get_value(name.clone()),
          group.unit,
          width = longest.unwrap(),
        )?;
      }
    }
    Ok(())
  }
}
