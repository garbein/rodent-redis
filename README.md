# rodent-redis

`rodent-redis` is an incomplete, idiomatic implementation of a
[Redis](https://redis.io) client and server built with
[async-std](https://async.rs).

The intent of this project is to learn rust and async-std.

**Disclaimer** Don't even think about trying to use this in production... just
don't.

## Running

The repository provides a server and a client.

### build

```
git clone https://github.com/garbein/rodent-redis.git
cd rodent-redis
cargo build --release

```

### Run Server

```
./target/release/rodent-redis-server
```

### Run Client

```
./target/release/rodent-redis-cli
```

```
127.0.0.1:6380> set key value
"OK"
127.0.0.1:6380> get key
value
127.0.0.1:6380> 
```

## Supported commands

`rodent-redis` currently only supports the following commands.

* [get](https://redis.io/commands/get)
* [set](https://redis.io/commands/set)
* [lpush](https://redis.io/commands/publish)
* [rpop](https://redis.io/commands/subscribe)

## Thanks
* [Redis](https://redis.io)
* [mini-redis](https://github.com/tokio-rs/mini-redis)
* [async-std](https://async.rs)