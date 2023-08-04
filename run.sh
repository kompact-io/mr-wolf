#!/usr/bin/env bash

cargo build --release --bin mr-wolf-server
cargo build --release --bin mr-wolf-client

./target/release/mr-wolf-server &> /dev/null &
server_id=$ID
(
  for ii in $(seq $1) 
  do
    ./target/release/mr-wolf-client &
  done

  wait
)

pkill -P $$
