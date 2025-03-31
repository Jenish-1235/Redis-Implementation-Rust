#!/bin/bash

# Apply kernel optimizations for high-performance networking
sysctl -w net.core.rmem_max=268435456
sysctl -w net.core.wmem_max=268435456
sysctl -w net.ipv4.tcp_rmem="4096 87380 268435456"
sysctl -w net.ipv4.tcp_wmem="4096 65536 268435456"
sysctl -w net.core.somaxconn=65535
sysctl -w net.ipv4.tcp_tw_reuse=1
sysctl -w net.ipv4.tcp_fin_timeout=30
sysctl -w net.ipv4.tcp_keepalive_time=600
sysctl -w net.ipv4.tcp_keepalive_intvl=30
sysctl -w net.ipv4.tcp_keepalive_probes=3


# Ensure jemalloc is installed
if ! ldconfig -p | grep -q libjemalloc; then
    apt-get update && apt-get install -y libjemalloc2
fi

# Set jemalloc as default memory allocator
export LD_PRELOAD="/usr/lib/x86_64-linux-gnu/libjemalloc.so"

# Increase file descriptors
ulimit -n 1000000
ulimit -u 1000000

exec "$@"
