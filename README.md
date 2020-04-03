# Wait-For-Them
Waits until all provided host and port pairs are opened.
It is written using async rust.

![Cast](/wait-for-them.gif)


## Installation

There are currently two way how to install the app.

You can install the binary only with a minimal subset of features.
```bash
cargo install wait-for-them
```

Or you can install it with all its features (including the nice progressbars).
```bash
cargo install wait-for-them --all-features
```

## Running

### Wait forever
```bash
wait-for-them host1:port1 host2:port2
```

### Wait with timeout (in miliseconds)
```
wait-for-them -t 5000 host1:port1 host2:port2
```

Note that it returns a number of unopened host:port combinations.
So if it worked ok it returns standard `0`.

### Execute a command after all hosts have opened ports
```
wait-for-them host1:port1 host2:port2 -- cmd arg1 arg2
```

Note that if the ports are opened it returns the status code of cmd.

## Motivation
The main motivation of this program was to use it within `docker-compose` config file (see `docker-compose.yml`).
To support waiting for multiple hostname:port records in parallel.
