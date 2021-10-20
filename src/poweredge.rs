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
