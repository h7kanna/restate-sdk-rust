# Restate Rust SDK

```shell
curl --http2-prior-knowledge localhost:3000/discover -XPOST
curl --http2-prior-knowledge localhost:3000/greet -XPOST -d "hello world"

curl --http2-prior-knowledge localhost:3000/invoke/Greeter/greet2 -XPOST -d "hello world"
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

curl -v localhost:8080/Greeter/greet -H 'content-type: application/json' -d '"Hi"'
```

```shell
restate invocations list
restate invocations cancel --yes inv_1edrEWkJnmse6PRDAW8yRiHQpnsgFfLaal
```