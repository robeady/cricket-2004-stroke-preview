pub struct Stroke {
    pub timings: [StrokeTiming; 5],
}

pub struct StrokeTiming {
    pub vertical: f64,
    pub direction: f64,
    pub direction_area: f64,
    pub power: f64,
    pub power_area: f64,
}
