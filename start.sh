cargo build
LD_PRELOAD=/usr/lib/gcc/x86_64-linux-gnu/9/libasan.so ./target/debug/j-fuzzer
