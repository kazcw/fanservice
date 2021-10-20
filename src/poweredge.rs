use crate::backend::{Backend, Result};
use ipmiraw::si::Ipmi;
use log::{error, trace};
use std::fmt::Debug;

pub struct PowerEdge {
    ipmi: Ipmi,
}
impl Backend for PowerEdge {
    fn new() -> Result<Self> {
        let ipmi = Ipmi::open("/dev/ipmi0").map_err(|e| Box::new(e) as Box<dyn Debug>)?;
        ipmi.cmd(0x30, 0x30, &mut [0x01, 0x00])
            .map_err(|e| Box::new(e) as Box<dyn Debug>)?;
        Ok(PowerEdge { ipmi })
    }

    fn set_speed(&mut self, speed: f64) -> Result<()> {
        assert!(speed >= 0.0);
        assert!(speed <= 1.0);
        let speed = (speed * 100.0).round() as u8;
        self.ipmi
            .cmd(0x30, 0x30, &mut [0x02, 0xff, speed])
            .map_err(|e| Box::new(e) as Box<dyn Debug>)
    }
}

impl Drop for PowerEdge {
    fn drop(&mut self) {
        trace!("restoring hardware fan control");
        let result = self
            .ipmi
            .cmd(0x30, 0x30, &mut [0x01, 0x01])
            .map_err(|e| Box::new(e) as Box<dyn Debug>);
        if let Err(e) = result {
            error!("Failed to restore hardware fan control! {:?}", e);
        }
    }
}
