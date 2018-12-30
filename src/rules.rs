use crate::model::Rules;

impl Rules {
    pub fn tick_time_interval(&self) -> f64 {
        1.0 / self.TICKS_PER_SECOND as f64
    }

    pub fn mean_e(&self) -> f64 {
        (self.MIN_HIT_E + self.MAX_HIT_E) / 2.0
    }

//    pub fn micro_tick_time_interval(&self) -> f64 {
//        self.tick_time_interval() / self.MICROTICKS_PER_TICK as f64
//    }
}
