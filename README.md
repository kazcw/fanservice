# fanservice

Daemon that regulates fan speeds based on temperature sensors. Supports Dell
PowerEdge server hardware.

## Usage

### Starting the daemon

First you need to start the daemon. You can start it manually, with something like:
```
sudo fanservice run -b poweredge -S /tmp/fanservice.socket
```
(`fanservice` must be run as root because it needs access to IPMI.)

But you'll probably want to run it as a system service. See the example
[systemd unit file](support/fanservice.service).

### Controlling the daemon

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
