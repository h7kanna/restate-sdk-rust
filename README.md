# Restate Rust SDK

```shell
curl --http2-prior-knowledge localhost:3000/discover -XPOST
curl --http2-prior-knowledge localhost:3000/greet -XPOST -d "hello world
```

```shell
cargo +nightly fmt
```

```shell
cargo test -- --nocapture
```

## Restate

```shell
restate-server
```

## Invocation

```shell
restate dp add http://localhost:3000

curl localhost:8080/Greeter/greet -H 'content-type: application/json' -d '"Hi"'
```