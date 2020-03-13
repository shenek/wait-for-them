# Wait-For-Them
Waits until all combination of hosts and ports are opened.

## Installation

```bash
cargo install wait-for-them
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