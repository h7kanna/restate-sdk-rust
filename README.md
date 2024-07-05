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

cargo test test_query -- --nocapture
```

## Restate

```shell
restate-server
```

## Invocation

```shell
restate dp add --yes http://localhost:3000 

restate dp add --yes --force http://localhost:3000

curl -v localhost:8080/greeter/greet -H 'content-type: application/json' -d '"world"'

curl -v localhost:8080/Greeter/greet -H 'content-type: application/json' -d '{"test": "test"}'

curl -v localhost:8080/Service/service -H 'content-type: application/json' -d '{"test": "test"}'

curl -v localhost:8080/Service/service -H 'content-type: application/json' -H 'idempotency-key: example' -d '{"test": "test"}'

curl -v localhost:8080/restate/awakeables/prom_1jcYC6A0foVMBkHaj7q8js_G8sHZwlBtzAAAAAQ/resolve -H 'content-type: application/json' -d '{"test": "next"}'

curl -v localhost:8080/ObjectService/workflow1/increment -H 'content-type: application/json' -d '{"test": "test"}'

curl -v localhost:8080/ObjectService/counter/increment -H 'content-type: application/json' -d '{"value": "100"}'

curl -v localhost:8080/ObjectService/counter/count -H 'content-type: application/json' -d '{"value": "100"}'

curl -v localhost:8080/WorkflowService/workflow1/run -H 'content-type: application/json' -d '{"test": "test"}'

curl -v localhost:8080/WorkflowService/workflow1/signal -H 'content-type: application/json' -d '{"test": "await_user1"}'

curl -v localhost:8080/WorkflowService/workflow1/signal -H 'content-type: application/json' -d '{"test": "await_user2"}'

curl -v localhost:8080/restate/workflow/WorkflowService/workflow1/attach

curl -v localhost:8080/restate/workflow/WorkflowService/workflow1/output

```

```shell
restate invocations list
restate invocations describe inv_1i8DzzOL6iN178sv2hRPz1aZ32v3lNyhWh
restate invocations cancel --yes inv_1gSomuaZJqT10TzzSN1nTOFycg8aggtPrj

restate sql "SELECT * FROM sys_journal sj WHERE sj.id = 'inv_1i8DzzOL6iN17u6DCDTNaXM5CXyZcRABXP' ORDER BY index LIMIT 100"

```

```shell

curl -v localhost:9070/query -H 'content-type: application/json' -d "{ \"query\" : \"SELECT * FROM sys_journal sj WHERE sj.id = 'inv_1i7mrFgVA6eD1gnGo4q57KH9dzPZwZuBAl' ORDER BY index LIMIT 1000\" }" -o journal

```

