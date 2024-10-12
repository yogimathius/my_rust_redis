# my_redis_server

## Description

`my_redis_server` is a Redis-compatible server implementation written in Rust. It provides a data structure server that handles data sent via TCP sockets using the Redis protocol. This server is designed to work with the `my_redis_client` for communication.

## Features

This server implements a wide range of Redis commands across various data types:

### Common Commands

- [x] `ECHO` – Echo the given string
- [x] `PING` – Test if server is responsive
- [x] `FLUSHALL` – Remove all keys from all databases

### Key/Value Commands

- [x] `SET` – Set key to hold the string value
- [x] `GET` – Get the value of key
- [x] `KEYS` – Find all keys matching the specified pattern
- [x] `TYPE` – Determine the type stored at key
- [x] `DEL` – Delete a key
- [x] `UNLINK` – Remove a key asynchronously in another thread
- [x] `EXPIRE` – Set a key's time to live in seconds
- [x] `RENAME` – Rename a key

### List Commands

- [x] `LLEN` – Get the length of a list
- [x] `LREM` – Remove elements from a list
- [x] `LINDEX` – Get an element from a list by its index
- [x] `LPOP/RPOP` – Remove and get the first/last element in a list
- [x] `LPUSH/RPUSH` – Prepend/Append one or multiple elements to a list
- [x] `LSET` – Set the value of an element in a list by its index

### Hash Commands

- [x] `HGET` – Get the value of a hash field
- [x] `HEXISTS` – Determine if a hash field exists
- [x] `HDEL` – Delete one or more hash fields
- [x] `HGETALL` – Get all the fields and values in a hash
- [x] `HKEYS` – Get all the fields in a hash
- [x] `HLEN` – Get the number of fields in a hash
- [x] `HMSET` – Set multiple hash fields to multiple values
- [x] `HSET` – Set the string value of a hash field
- [x] `HVALS` – Get all the values in a hash

## Requirements

- [x] Rust (latest stable version)
- [x] Tokio
- [x] Bytes
- [x] Structopt

## Installation

To install and run `my_redis_server` locally, ensure you have Rust installed. Then clone the repository and build the project:

```bash
git clone https://your-repository-url/my_redis_server.git
cd my_redis_server
cargo build --release
```

## Usage

To start the server, use the following command:

```bash
cargo run --bin my_redis_server [--port PORT]
```

The default port is 6379 if not specified.

## Architecture

### Atomic Operations

The server ensures that multiple clients can modify the same data simultaneously without conflicts.

### Persistence

The server automatically saves its database every 300 seconds into a file named `dump.rdb` in the current directory. Backups are performed as a background task.

### Shutdown

The server catches the Ctrl+C signal to shut down correctly, saving the database before exiting.

### Data Storage

The server uses a custom data structure based on HashMaps for efficient data storage and retrieval.

### Connections

The server uses Tokio's TcpListener to handle multiple client connections simultaneously.

### Protocol

The server implements the Redis wire protocol for communication with clients.

## Command Reference

For more information on Redis commands, refer to the [Redis Command Reference](https://redis.io/commands).

## Author

Mathius Johnson
