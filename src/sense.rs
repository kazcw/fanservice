use sensors::Sensors;

#[derive(Debug)]
pub struct Sense {
    //chip: &'static str,
    //name: String,
    max: f64,
    //crit: f64,
    getcur: sensors::Subfeature,
}

impl Sense {
    pub fn excess(&self) -> f64 {
        self.getcur.get_value().unwrap() - self.max
    }
}

pub fn get_sensors() -> Vec<Sense> {
    let sensors = Sensors::new();
    let mut feats = vec![];
    for chip in sensors {
        //let chipname = Box::leak(chip.get_name().unwrap().into_boxed_str());
        for feature in chip {
            //let name = feature.get_label().unwrap();
            let mut max = None;
            //let mut crit = None;
            let mut getcur = None;
            for subfeature in feature {
                let name = subfeature.name();
                if name.ends_with("_max") {
                    max = Some(subfeature.get_value().unwrap())
                //} else if name.ends_with("_crit") {
                //    crit = Some(subfeature.get_value().unwrap())
                } else if name.ends_with("_input") {
                    getcur = Some(subfeature)
                }
            }
            feats.push(Sense {
                //chip: chipname,
                //name: name.to_owned(),
                max: max.unwrap(),
                //crit: crit.unwrap(),
                getcur: getcur.unwrap(),
            });
        }
    }
    feats
}
