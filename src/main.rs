#[cfg(feature = "ipmi")]
use crate::poweredge::PowerEdge;
use backend::{Backend, DryRun};
use clap::{App, AppSettings, Arg, SubCommand};
use client::Client;
use daemon::Daemon;

mod client;
mod daemon;
#[cfg(feature = "ipmi")]
mod poweredge;
mod sense;

mod backend {
    use std::fmt::Debug;

    pub type Error = Box<dyn Debug>;
    pub type Result<T> = std::result::Result<T, Error>;
    pub trait Backend {
        fn new() -> Result<Self>
        where
            Self: Sized;
        fn set_speed(&mut self, speed: f64) -> Result<()>;
    }

    pub struct DryRun;
    impl Backend for DryRun {
        fn new() -> Result<Self> {
            Ok(DryRun)
        }
        fn set_speed(&mut self, _: f64) -> Result<()> {
            Ok(())
        }
    }
}

mod protocol {
    use serde::{Deserialize, Serialize};
    pub use std::os::unix::net::UnixDatagram as Socket;

    #[derive(Serialize, Deserialize, Debug)]
    pub enum Message {
        SetGamma(f64),
    }
}

fn main() {
    cfg_if::cfg_if! {
        if #[cfg(feature = "systemd")] {
            systemd_journal_logger::init().unwrap();
            log::set_max_level(log::LevelFilter::Info);
        } else if #[cfg(feature = "env_logger")] {
            env_logger::init();
        }
    }

    let args = App::new("fanservice")
        .setting(AppSettings::SubcommandRequiredElseHelp)
        .version("0.3.0")
        .author("Kaz Wesley <fanservice@lambdaverse.org>")
        .about("Temperature-sensor based fan-speed regulator for PowerEdge servers")
        .arg(
            Arg::with_name("socket")
                .short("S")
                .long("socket")
                .help("Path to control socket")
                .default_value("/run/fanservice/control"),
        )
        .subcommand(
            SubCommand::with_name("run")
                .help("run daemon in foreground")
                .arg(
                    Arg::with_name("quiet-factor")
                        .short("q")
                        .long("quiet-factor")
                        .help("quiet factor (0-1: aggressive cooling, >1 laxer cooling)")
                        .default_value("1.0"),
                )
                .arg(
                    Arg::with_name("backend")
                        .short("b")
                        .long("backend")
                        .required(true)
                        .takes_value(true),
                ),
        )
        .subcommand(
            SubCommand::with_name("set")
                .help("connect to daemon and apply new settings for its parameters")
                .arg(
                    Arg::with_name("quiet-factor")
                        .short("q")
                        .long("quiet-factor")
                        .help("quiet factor (0-1: aggressive cooling, >1 laxer cooling)")
                        .takes_value(true),
                ),
        )
        .get_matches();

    let socket = args.value_of("socket").unwrap();

    match args.subcommand() {
        ("run", Some(args)) => {
            let gamma = args
                .value_of("quiet-factor")
                .unwrap()
                .parse()
                .expect("quiet-factor must be a number");
            let backend = args.value_of("backend").unwrap();
            let backend = match backend {
                #[cfg(feature = "ipmi")]
                "poweredge" => Box::new(PowerEdge::new().unwrap()) as Box<dyn Backend>,
                "dryrun" => Box::new(DryRun::new().unwrap()) as Box<dyn Backend>,
                x => panic!("unsupported backend: {:?}", x),
            };
            Daemon::new(socket, backend, gamma).unwrap().run();
        }
        ("set", Some(args)) => {
            let gamma = args
                .value_of("quiet-factor")
                .map(|x| x.parse().expect("quiet-factor must be a number"));
            let client = Client::connect(socket).unwrap();
            if let Some(gamma) = gamma {
                client.set_gamma(gamma).unwrap();
            }
        }
        _ => unreachable!(),
    }
}
