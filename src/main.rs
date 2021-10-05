// lm_sensors stuff

use sensors::Sensors;

#[derive(Debug)]
struct Feature {
    chip: &'static str,
    name: String,
    max: f64,
    crit: f64,
    getcur: sensors::Subfeature,
}

impl Feature {
    fn excess(&self) -> f64 {
        self.getcur.get_value().unwrap() - self.max
    }
}

fn get_feats() -> Vec<Feature> {
    let sensors = Sensors::new();
    let mut feats = vec![];
    for chip in sensors {
        let chipname = Box::leak(chip.get_name().unwrap().into_boxed_str());
        for feature in chip {
            let name = feature.get_label().unwrap();
            let mut max = None;
            let mut crit = None;
            let mut getcur = None;
            for subfeature in feature {
                let name = subfeature.name();
                if name.ends_with("_max") {
                    max = Some(subfeature.get_value().unwrap())
                } else if name.ends_with("_crit") {
                    crit = Some(subfeature.get_value().unwrap())
                } else if name.ends_with("_input") {
                    getcur = Some(subfeature)
                }
            }
            feats.push(Feature {
                chip: chipname,
                name: name.to_owned(),
                max: max.unwrap(),
                crit: crit.unwrap(),
                getcur: getcur.unwrap(),
            });
        }
    }
    feats
}

// IPMI stuff

pub trait Backend {
    type Error;
    fn new() -> Result<Self, Self::Error> where Self: Sized;
    fn set_speed(&mut self, speed: u32) -> Result<(), Self::Error>;
}

mod poweredge {
    use crate::Backend;
    use ipmiraw::si::Ipmi;
    use std::convert::TryInto;

    pub struct Poweredge {
        ipmi: Ipmi,
    }
    impl Backend for Poweredge {
        type Error = ();
        fn new() -> Result<Self, Self::Error> {
            let ipmi = Ipmi::open("/dev/ipmi0").unwrap();
            ipmi.cmd(0x30, 0x30, &mut [0x01, 0x00]);
            Ok(Poweredge { ipmi })
        }

        fn set_speed(&mut self, speed: u32) -> Result<(), Self::Error> {
            assert!(speed <= 100);
            let speed = speed.try_into().unwrap();
            self.ipmi.cmd(0x30, 0x30, &mut [0x02, 0xff, speed]);
            Ok(())
        }
    }
}
use poweredge::Poweredge;

// core logic

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

fn main() {
    const INTERVAL_MS: u64 = 2_000;
    const SMOOTHNESS: usize = 10;
    let period = std::time::Duration::from_millis(INTERVAL_MS);
    let mut worsts = vec![std::f64::MIN; SMOOTHNESS];
    let mut i = 0;
    let feats = get_feats();
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
