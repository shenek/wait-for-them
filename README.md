![Security audit](https://github.com/shenek/wait-for-them/workflows/Security%20audit/badge.svg)
![Code Quality](https://github.com/shenek/wait-for-them/workflows/Code%20Quality/badge.svg)
![Release](https://github.com/shenek/wait-for-them/workflows/Release/badge.svg)
[![Documentation](https://docs.rs/wait-for-them/badge.svg)](https://docs.rs/wait-for-them/)
![Downloads](https://img.shields.io/crates/d/wait-for-them.svg)

# Wait-For-Them
Waits until all provided host and port pairs are opened or return status 200 in case of http(s) url.
It is written using async rust.

![Cast](/wait-for-them.gif)


## Installation

There are currently two way how to install the app.

You can install the binary only with a minimal subset of features.
```bash
cargo install wait-for-them --no-default-features
```

Or you can install it with all its features (including the nice progressbars and http(s) support).
```bash
cargo install wait-for-them
```

## Running

### Wait forever
```bash
wait-for-them host1:port1 host2:port2 http://host3:8080/
```

### Wait with timeout (in milliseconds)
```
wait-for-them -t 5000 host1:port1 host2:port2 http://host3:8080/
```

Note that it returns a number of unopened host:port combinations.
So if it worked ok it returns standard `0`.

### Execute a command after all hosts have opened ports
```
wait-for-them host1:port1 host2:port2 http://host3:8080/ -- cmd arg1 arg2
```

Note that if the ports are opened it returns the status code of cmd.

## Motivation
The main motivation of this program was to use it within `docker-compose` config file (see `docker-compose.yml`).
To support waiting for multiple hostname:port records in parallel.
