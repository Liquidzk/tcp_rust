#!/bin/bash

cargo b --release
ext=$?
if [[ $ext -ne 0 ]]; then
    exit $ext
fi

sudo setcap cap_net_admin=eip target/release/tcp_rust
target/release/tcp_rust &
pid=$!

sudo ip addr add dev tun0 192.168.0.1/24
sudo ip link set up dev tun0
trap "kill ${pid}" INT TERM
wait $pid