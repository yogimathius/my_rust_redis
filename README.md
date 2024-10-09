# my_redis_server

## NAME

**my_redis_server**  
--port PORT (Default port: 6379)

---

## Prologue

Redis’s exceptional performance, simplicity, and atomic manipulation of data structures lends itself to solving problems that are difficult or perform poorly when implemented with traditional relational databases. Below are some example use cases:

### COMMON USE CASES

- **Caching**: Redis is widely used for caching due to its performance and ability to persist data to disk. It is a superior alternative to memcached for caching scenarios with a high volume of read and write operations.
- **Publish and Subscribe**: Since version 2.0, Redis has supported the Publish/Subscribe messaging paradigm, which some organizations use as a simpler and faster alternative to traditional message queuing systems like zeromq and RabbitMQ.

- **Queues**: Redis is used in projects such as Resque for queueing background jobs.

- **Counters**: Atomic commands such as `HINCRBY` allow for simple and thread-safe counters without needing to read data before incrementing or updating database schemas.

---

## DESCRIPTION

You are tasked with building **my_redis_server**, a data structure server that provides access to data sent via TCP sockets using the Redis protocol. You should use the previously implemented **my_redis_client** to communicate with **my_redis_server**.

Redis supports two primary data types:

**Lists**

```bash
$> ./my_redis_server &
$> ./my_redis_client
127.0.0.1:6379> LPUSH my_list 1
(integer) 1
127.0.0.1:6379> LPUSH my_list 2
(integer) 2
127.0.0.1:6379> SET "hello" "world"
OK
127.0.0.1:6379> GET "hello"
"world"
```

**Hashes (k/v)**

```bash
$>./my_redis_server &
$>./my_redis_client
127.0.0.1:6379>set "hello" "world"
OK
127.0.0.1:6379>get "hello"
"world"
127.0.0.1:6379>
```

## Commands

In order to mirror your my_redis_client, we will implement the followings commands:

### Common

- echo
- ping
- flushall

### Key/Value

- set
- get
- keys
- type
- del
- unlink
- expire
- rename

### Lists

- llen
- lrem
- lindex
- lpop/rpop
- lpush/rpush
- lset

### Hashes

- hget
- hexists
- hdel
- hgetall
- hkeys
- hlen
- hmset
- hset
- hvals

### Encryption

my_redis_server should use a plain TCP connection.

### Connection

my_redis_server must simultaneously accept multiple connections.

## Persistence/backup

my_redis_server will save its database every 300 seconds into a file. This file will be located in the current directory and it will be named: dump.my_rdb.

RDB format is a binary representation of the memory in a redis-server. You can decide to follow its implementation or implement your own backup format.

Backups must happen as a background task.

Db
my_redis_server must provide 1 database.

## Recommended libraries

- atoi
- Tokio
- Bytes
- structopt
  !! redis-rs is obviously not authorized. You can`t use this library.

## Technical description

You will provide a Cargo-compatible project. Consider the following 7 items:

### Atomic

Multiple clients can modify the same data at the same time. You will have to make sure this works.

### Shutdown correctly

Signal ctrl_c must be caught in order to correctly shutdown the server. Before shutting down, you will save into the dump.my_rdb file.

### Data storage

A custom data structure would be preferable but you can use vec and hashmap for data storage.

### Arguments

Use structopt

### Sockets

Use tokio TcpListener

### Protocol

The Redis wire protocol documentation can be found here. Its implementation should be simple.

### Design/Architecture

You should reuse your protocols and connections structs from my_redis_client. At that moment, you will either proud of your previous implementation or it will be time to refactor it.
