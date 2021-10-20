use crate::backend::Backend;
use crate::protocol::{Message, Socket};
use log::{debug, error, info, trace, warn};
use std::fs;
use std::io;

#[derive(Debug)]
pub enum Error {
    Io(io::Error),
}
pub type Result<T> = std::result::Result<T, Error>;

impl From<io::Error> for Error {
    fn from(e: io::Error) -> Self {
        Error::Io(e)
    }
}

pub struct Daemon {
    sock: Socket,
    gamma_table: [f32; 17],
    backend: Box<dyn Backend>,
    msg_buf: Vec<u8>,
    sock_path: String,
}

// copied from standard library because it's not stable (yet?)
fn lerp(x: f64, start: f64, end: f64) -> f64 {
    if start == end {
        start
    } else {
        x.mul_add(end, (-x).mul_add(start, start))
    }
}

impl Daemon {
    pub fn new(sock_path: &str, backend: Box<dyn Backend>, gamma: f64) -> Result<Self> {
        let _ = fs::remove_file(sock_path);
        let sock = Socket::bind(sock_path)?;
        sock.set_nonblocking(true)?;
        let msg_buf = vec![0; 64];
        let sock_path = sock_path.to_owned();
        let gamma_table = [0.0; 17];
        let mut daemon = Self {
            sock,
            backend,
            msg_buf,
            sock_path,
            gamma_table,
        };
        daemon.set_gamma(gamma);
        Ok(daemon)
    }

    fn set_gamma(&mut self, gamma: f64) {
        for (i, y) in self.gamma_table.iter_mut().enumerate() {
            *y = (i as f64 / 16.0).powf(gamma) as f32;
            debug!("gamma_table[{}] = {}", i, *y);
        }
    }

    fn apply_gamma(&self, x: f64) -> f64 {
        let t = x * 16.0;
        let t0 = t.floor();
        let t1 = t.ceil();
        let dx = t - t0;
        let g0 = self.gamma_table[t0 as usize].into();
        let g1 = self.gamma_table[t1 as usize].into();
        lerp(dx, g0, g1)
    }

    fn speed_factor_for_excess(&self, excess: f64) -> f64 {
        // degC below rated max temp to reach 100% fan speed
        const ACTION_MARGIN: f64 = 5.0;
        // temp difference between min and max fan speeds
        const RANGE: f64 = 35.0;

        ((excess + ACTION_MARGIN + RANGE) / RANGE).clamp(0.0, 1.0)
    }

    fn speed_for_excess(&self, excess: f64) -> f64 {
        // min fan factor (0-1)
        const MIN_FAN: f64 = 0.10;

        let sf = self.speed_factor_for_excess(excess);

        let sf = self.apply_gamma(sf);

        MIN_FAN + (sf * (1.0 - MIN_FAN))
    }

    fn handle_messages(&mut self) {
        loop {
            let r = self.sock.recv(&mut self.msg_buf);
            let n = match r {
                Ok(n) => n,
                Err(e) => {
                    if e.kind() != io::ErrorKind::WouldBlock {
                        error!("failed to check for messages! {:?}", e)
                    }
                    break;
                }
            };
            let m = bincode::deserialize(&self.msg_buf[..n]);
            let m = match m {
                Ok(m) => m,
                Err(e) => {
                    error!("deserializing client message: {:?}", e);
                    continue;
                }
            };
            info!("received client message: {:?}", m);
            match m {
                Message::SetGamma(g) => self.set_gamma(g),
            }
        }
    }

    // TODO: make all the constants configurable
    // (config file // command line // client)

    pub fn run(mut self) {
        const INTERVAL_MS: u64 = 2_000;
        const SMOOTHNESS: usize = 10;
        let period = std::time::Duration::from_millis(INTERVAL_MS);
        let mut worsts = vec![std::f64::MIN; SMOOTHNESS];
        let mut i = 0;
        let feats = crate::sense::get_sensors();
        loop {
            self.handle_messages();
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
            let speed = self.speed_for_excess(excess);
            debug!("set speed: {}", speed);
            self.backend.set_speed(speed).unwrap();
            std::thread::sleep(period);
        }
    }
}

impl Drop for Daemon {
    fn drop(&mut self) {
        let result = fs::remove_file(&self.sock_path);
        if let Err(e) = result {
            warn!("couldn't delete socket before exiting: {:?}", e);
        }
    }
}
