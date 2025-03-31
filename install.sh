#!/bin/bash

# Apply kernel optimizations for high-performance networking
sysctl -w \
net.core.rmem_max=268435456 \
net.core.wmem_max=268435456 \
net.ipv4.tcp_rmem="4096 87380 268435456" \
net.ipv4.tcp_wmem="4096 65536 268435456" \
net.core.somaxconn=65535 \
net.ipv4.tcp_tw_reuse=1 \
net.ipv4.tcp_fin_timeout=30 \
net.ipv4.tcp_keepalive_time=600 \
net.ipv4.tcp_keepalive_intvl=30 \
net.ipv4.tcp_keepalive_probes=3

# Load new sysctl settings
sysctl -p

# Enable jemalloc
export LD_PRELOAD="/usr/lib/x86_64-linux-gnu/libjemalloc.so"

# Start the server with proper resource limits
ulimit -n 1000000
exec "$@"