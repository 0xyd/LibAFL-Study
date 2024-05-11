# Use LibAFL to fuzz a python harness 

## Prerequisites

1. ZMQ (Ububtu: `apt-get install libzmq3-dev`)
1. `pip install pyzmq`
1. Put the following in the `Cargo.toml` for rust: (Note: version `0.10.5` suggested by official website does not working for me.)
```rust
[dependencies]
zmq = "0.10.0"
```




