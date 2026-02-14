# tt-metal-bindings

Rust FFI bindings for Tenstorrent's tt-metal library.

## Quick Start

```bash
# 1. Build tt-metal first (in ../tt-metal)
cd ../tt-metal && ./build_metal.sh

# 2. Set environment and run
cd ../tt-metal-rs
export TT_METAL_HOME=/home/your_home_user/tt-metal
export TT_METAL_RUNTIME_ROOT=$TT_METAL_HOME
export LD_LIBRARY_PATH="$TT_METAL_HOME/build/tt_metal:$TT_METAL_HOME/build/lib:$LD_LIBRARY_PATH"
export TT_METAL_SLOW_DISPATCH_MODE=1

TT_METAL_SLOW_DISPATCH_MODE=1 cargo run --example hello_kernel
```


