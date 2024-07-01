# Restate Rust SDK

```shell
curl --http2-prior-knowledge localhost:3000/discover
curl --http2-prior-knowledge localhost:3000/greet -XPOST -d "hello world"

curl --http2-prior-knowledge localhost:3000/invoke/Greeter/greet2 -XPOST -d "hello world"
```

```shell
cargo +nightly fmt
```

```shell
cargo test test_handle_connection -- --nocapture
```

## Restate

```shell
restate-server
```

## Invocation

```shell
restate dp add --yes http://localhost:3000 

restate dp add --yes --force http://localhost:3000

curl -v localhost:8080/Greeter/greet -H 'content-type: application/json' -d '{"test": "test"}'

curl -v localhost:8080/Service/service -H 'content-type: application/json' -d '{"test": "test"}'

curl -v localhost:8080/Service/service -H 'content-type: application/json' -H 'idempotency-key: example' -d '{"test": "test"}'

curl -v localhost:8080/ObjectService/workflow1/increment -H 'content-type: application/json' -d '{"test": "test"}'

curl -v localhost:8080/ObjectService/workflow1/count -H 'content-type: application/json' -d '{"test": "test"}'

curl -v localhost:8080/WorkflowService/workflow1/run -H 'content-type: application/json' -d '{"test": "test"}'

curl -v localhost:8080/restate/awakeables/prom_15-4e7a6rR9MBkG8D6GBJYiYA4ZMbQyekAAAAAQ/resolve -H 'content-type: application/json' -d '{"test": "next"}'
```

```shell
restate invocations list
restate invocations describe inv_1i8DzzOL6iN178sv2hRPz1aZ32v3lNyhWh
restate invocations cancel --yes inv_1gSomuaZJqT10TzzSN1nTOFycg8aggtPrj
```

```shell

curl -v localhost:9070/query -H 'content-type: application/json' -d "{ \"query\" : \"SELECT * FROM sys_journal sj WHERE sj.id = 'inv_1b3ZbyPDEvc85opwOyw39cwtduHjN88ZUt' ORDER BY index LIMIT 1000\" }"

```

