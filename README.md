[![progress-banner](https://backend.codecrafters.io/progress/redis/c67e6872-8dd6-46e2-9bdb-d404b04efea8)](https://app.codecrafters.io/users/codecrafters-bot?r=2qF)

This is a starting point for Rust solutions to the
["Build Your Own Redis" Challenge](https://codecrafters.io/challenges/redis).

In this challenge, you'll build a toy Redis clone that's capable of handling
basic commands like `PING`, `SET` and `GET`. Along the way we'll learn about
event loops, the Redis protocol and more.

**Note**: If you're viewing this repo on GitHub, head over to
[codecrafters.io](https://codecrafters.io) to try the challenge.

# Passing the first stage

The entry point for your Redis implementation is in `src/main.rs`. Study and
uncomment the relevant code, and push your changes to pass the first stage:

```sh
git add .
git commit -m "pass 1st stage" # any msg
git push origin master
```

That's all!

# Stage 2 & beyond

Note: This section is for stages 2 and beyond.

1. Ensure you have `cargo (1.54)` installed locally
1. Run `./spawn_redis_server.sh` to run your Redis server, which is implemented
   in `src/main.rs`. This command compiles your Rust project, so it might be
   slow the first time you run it. Subsequent runs will be fast.
1. Commit your changes and run `git push origin master` to submit your solution
   to CodeCrafters. Test output will be streamed to your terminal.

### General Commands

- [x] `quit` – Exit the CLI
- [x] `ECHO` – Echoes a given message
- [x] `PING` – Checks if the server is alive
- [x] `FLUSHALL` – Clears all data on the Redis server
- [x] `INFO` – Fetches server information

### Key/Value Commands

- [x] `SET` – Set a key to hold a specific value
- [x] `GET` – Get the value of a key
- [ ] `KEYS` – List all keys matching a pattern
- [x] `TYPE` – Get the type of a key
- [x] `DEL` – Delete a key
- [x] `UNLINK` – Asynchronously delete a key
- [x] `EXPIRE` – Set a timeout on a key
- [ ] `RENAME` – Rename a key

### List Commands

- [x] `LLEN` – Get the length of a list
- [ ] `LREM` – Remove elements from a list
- [ ] `LINDEX` – Get an element from a list by its index
- [ ] `LPOP/RPOP` – Remove and return the first/last element from a list
- [ ] `LPUSH/RPUSH` – Prepend/Append an element to a list
- [x] `LSET` – Set the value of an element in a list by its index

### Hash Commands

- [ ] `HGET` – Get the value of a hash field
- [ ] `HEXISTS` – Check if a hash field exists
- [ ] `HDEL` – Delete one or more hash fields
- [ ] `HGETALL` – Get all fields and values of a hash
- [ ] `HKEYS` – Get all field names in a hash
- [ ] `HLEN` – Get the number of fields in a hash
- [ ] `HMSET` – Set multiple fields in a hash
- [ ] `HSET` – Set a field in a hash
- [ ] `HVALS` – Get all values in a hash
