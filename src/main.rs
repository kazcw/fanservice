mod poweredge;
mod sense;

pub trait Backend {
    type Error;
    fn new() -> Result<Self, Self::Error>
    where
        Self: Sized;
    fn set_speed(&mut self, speed: u32) -> Result<(), Self::Error>;
}

mod daemon {
    use crate::poweredge::Poweredge;
    use crate::Backend;

    fn speed_for_excess(excess: f64) -> u32 {
        // degC below rated max temp to reach 100% fan speed
        const ACTION_MARGIN: f64 = 5.0;
        // min % to run fan
        const MIN_FAN: f64 = 10.0;
        // temp difference between min and max fan speeds
        const RANGE: f64 = 35.0;

        (MIN_FAN + ((excess + ACTION_MARGIN + RANGE) / RANGE * (100.0 - MIN_FAN)))
            .clamp(MIN_FAN, 100.0)
            .round() as u32
    }

    pub fn run() {
        const INTERVAL_MS: u64 = 2_000;
        const SMOOTHNESS: usize = 10;
        let period = std::time::Duration::from_millis(INTERVAL_MS);
        let mut worsts = vec![std::f64::MIN; SMOOTHNESS];
        let mut i = 0;
        let feats = crate::sense::get_sensors();
        let mut backend = Poweredge::new().unwrap();
        loop {
            worsts[i] = feats
                .iter()
                .map(|x| x.excess())
                .max_by(|x, y| x.partial_cmp(y).unwrap())
                .unwrap();
            i = (i + 1) % SMOOTHNESS;
            let excess = *worsts
                .iter()
                .max_by(|x, y| x.partial_cmp(y).unwrap())
                .unwrap();
            backend.set_speed(speed_for_excess(excess)).unwrap();
            std::thread::sleep(period);
        }
    }
}

fn main() {
    daemon::run();
}
