use crate::protocol::{Message, Socket};
use bincode;
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

pub struct Client {
    sock: Socket,
}

impl Client {
    pub fn connect(path: &str) -> Result<Self> {
        let sock = Socket::unbound()?;
        sock.connect(path)?;
        Ok(Self { sock })
    }

    pub fn set_gamma(&self, gamma: f64) -> Result<()> {
        let m = Message::SetGamma(gamma);
        let m = bincode::serialize(&m).unwrap();
        let _ = self.sock.send(&m)?;
        Ok(())
    }
}
