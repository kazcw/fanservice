[![Build Status](https://app.travis-ci.com/kazcw/jerbs.svg?branch=master)](https://app.travis-ci.com/kazcw/jerbs)

# fanservice

Daemon that regulates fan speeds based on temperature sensors; speed curves can
be tuned at runtime with a CLI tool. Supports Dell PowerEdge server hardware.

## Installing the daemon and CLI tool

If you use the **Nix** package manager, there's a package in [my
overlay](https://github.com/kazcw/phoe.nix); enable the service with
`services.fanservice.enable = true;` and add the `fanservice` package to your
user environment.

**Otherwise**, you can build `fanservice` with cargo:
```
$ cargo install fanservice
```

You'll probably want to run it as a system service. See the example
[systemd unit file](support/fanservice.service).

## Controlling the daemon with the CLI tool

Once your daemon is running, you can send it control messages. Let's try
turning up the quiet-factor a little:
```
fanservice set -q 1.3
```
(You must run the client command as a user who has access to the daemon's
socket file.)

`fanservice` always works to ensure all system temperatures are within
acceptable ranges, but within those ranges you have a choice of how
aggressively to keep the system cool.
- at quiet-factors below 1, the fans run more aggressively than at 1 (at `-q
  0`, they always run at 100%)
- at quiet-factor 1, the fans respond linearly to temperature
- at factors above 1, the fans don't run as loud unless the system gets hot
- at really high factors, the fans will run near minimum speed until
  temperatures reach the top of the acceptable range, and then they will
  quickly approach 100%

For some reference points, I use `-q 1.3` during the daytime, and `-q 1.8` when
I'm trying to sleep in the same room as my rack. You'll want to experiment and
see what works best for your climate, workload, and noise concerns.
