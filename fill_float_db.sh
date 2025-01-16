#!/bin/bash

# Command to run Cargo with your specified options
CARGO_CMD="/home/leo/.cargo/bin/cargo run --color=always --package rql --bin rql --profile dev"

# Start a subshell to collect all SQL commands
(
  # Loop 100 times to generate the INSERT INTO commands
  for i in $(seq 0 10000000); do
    # Generate three random floats using $RANDOM, bc, and printf to format to 6 decimal places
    FLOAT1=$(echo "scale=8; $RANDOM / 32767" | bc -l | awk '{printf "%.6f", $0}')
    FLOAT2=$(echo "scale=8; $RANDOM / 32767" | bc -l | awk '{printf "%.6f", $0}')
    FLOAT3=$(echo "scale=8; $RANDOM / 32767" | bc -l | awk '{printf "%.6f", $0}')

    # Echo the SQL command
    echo "INSERT INTO floats VALUES ($FLOAT1, $FLOAT2, $FLOAT3);"
  done
) | $CARGO_CMD  # Pipe all commands at once into Cargo