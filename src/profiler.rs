use std::time::Instant;

pub struct Profiler {
    start: Instant,
    stages: Vec<Stage>,
}

pub struct Stage {
    name: &'static str,
    finish: Instant,
}

#[derive(Serialize)]
pub struct Interval {
    name: &'static str,
    duration: f64,
}

impl Profiler {
    pub fn new(start: Instant) -> Self {
        Profiler { start, stages: Vec::new() }
    }

    pub fn reset(&mut self, start: Instant) {
        self.start = start;
        self.stages.clear();
    }

    pub fn stage(&mut self, name: &'static str, finish: Instant) {
        self.stages.push(Stage { name, finish })
    }

    pub fn report(&self) -> Vec<Interval> {
        use crate::my_strategy::common::milliseconds;

        let mut last = self.start;
        let mut result = Vec::with_capacity(self.stages.len());

        for stage in self.stages.iter() {
            result.push(Interval {
                name: stage.name,
                duration: milliseconds(stage.finish - last),
            });
            last = stage.finish;
        }

        result
    }
}
