# LibAFL-Study

LibAFL is sexy but I am just a muggle :) 

## Install

Although I am a muggle, I still know how to [install rust](https://www.rust-lang.org/tools/install). But if the rust is too old, the installation might be failed too. So, as what official document suggests, run `rustup upgrade` to update it.

1. Clone LibAFL repo. (Here I just add the main branch of it as the submodule.)
2. Enter the repo and use command `cargo build --release` to build it. If there is a complain about `failed to run custom build command for libafl_cc`. Try to install `libstdc++` (check [here](https://github.com/AFLplusplus/LibAFL/issues/1434))

## Use it

Put the following in the `Cargo.toml` for the fuzzing projects.
```cargo
[dependencies]
libafl = { version = "*" }
```

There are other cool stuff but at the moment I don't need them :)


## References
1. [The LibAFL Fuzzing Library](https://aflplus.plus/libafl-book/)