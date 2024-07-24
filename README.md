# Restate Rust SDK prototype

This is a prototype implementation of Rust SDK for [Restate](https://restate.dev/)

**Note**: This is a work in progress and in rapid development stage. Updates will be posted on
the [blog](https://content.denote.dev/let-us-design-a-durable-promise-sdk-in-rust/)

**DO NOT USE IN PRODUCTION**

## Planned features

- [] API
- [] Service
- [] State
- [] Virtual Object
- [] Workflow
- [] Awakeables
- [] Side Effect
- [] Logger

# Examples

## Restate

Run Restate

```shell
restate-server
```

Clean up

```shell
rm -rf restate-data
```

## Service

```
cd examples
```

```shell
cargo run --example <EXAMPLE> 
```

For instance, run the first example

```shell
cargo run --example example4
```

Before running each example

## Deployment and discovery

### Deployment

```shell
restate dp add --yes http://localhost:3000 
```

or

```shell
restate dp add --yes --force http://localhost:3000
```

### Discovery

```shell
curl --http2-prior-knowledge localhost:3000/discover
```

## Invocation

### Example 1

Basic service

```shell
cargo run --example example1
 
```

#### Invocations

```shell
curl -v localhost:8080/Service/service -H 'content-type: application/json' -d '{"test": "test"}'
```

### Example 2

Manual implementation of traits

### Example 3

Side effects (run)

```shell
cargo run --example example3
 
```

#### Invocations

```shell
curl -v localhost:8080/Service/service -H 'content-type: application/json' -d '{"test": "test"}'
```

### Example 4

External


### Example 5

Sleep and Awakeables

```shell
cargo run --example example5
 
```

#### Service

#### Invocations

```shell
curl -v localhost:8080/Service/service -H 'content-type: application/json' -d '{"test": "test"}'
curl -v localhost:8080/restate/awakeables/prom_1rWMkEHIGEbgBkOBswV1G4I9IjP9NpljTAAAAAQ/resolve -H 'content-type: application/json' -d '{"test": "next"}'

```

With idempotency-key

```shell
curl -v localhost:8080/Service/service -H 'content-type: application/json' -H 'idempotency-key: example' -d '{"test": "test"}'

```

### Example 6

Virtual objects and state

#### Service

```shell
cargo run --example object 
 
```

#### Invocations

```shell
curl -v localhost:8080/ObjectService/counter/increment -H 'content-type: application/json' -d '{"value": "100"}'

curl -v localhost:8080/ObjectService/counter/count -H 'content-type: application/json' -d '{"value": "100"}'
```

### Example 7

Workflows

#### Service

```shell
cargo run --example workflow
 
```

#### Invocations

```shell
curl -v localhost:8080/WorkflowService/workflow1/run -H 'content-type: application/json' -d '{"test": "test"}'

curl -v localhost:8080/WorkflowService/workflow1/signal -H 'content-type: application/json' -d '{"test": "await_user1"}'

curl -v localhost:8080/WorkflowService/workflow1/signal -H 'content-type: application/json' -d '{"test": "await_user2"}'

curl -v localhost:8080/WorkflowService/workflow1/signal -H 'content-type: application/json' -d '{"test": "await_user3"}'

curl -v localhost:8080/restate/workflow/WorkflowService/workflow1/attach

curl -v localhost:8080/restate/workflow/WorkflowService/workflow1/output
```

### Example 8

Combinators: TODO

#### Service

```shell
cargo run --example combinator
 
```

```shell
curl -v localhost:8080/Service/select -H 'content-type: application/json' -d '{"test": "test"}'
curl -v localhost:8080/Service/join -H 'content-type: application/json' -d '{"test": "test"}'
```

# Introspection and troubleshooting

```shell

restate invocations list
restate invocations describe inv_1i8DzzOL6iN178sv2hRPz1aZ32v3lNyhWh
restate invocations cancel --yes inv_1gSomuaZJqT10TzzSN1nTOFycg8aggtPrj

```

```shell
restate sql "SELECT * FROM sys_journal sj WHERE sj.id = 'inv_12fRY4rsDA5q1s8FL2KiWosO8gKLPxgPpT' ORDER BY index LIMIT 100"
```

```shell

curl -v localhost:9070/query -H 'content-type: application/json' -d "{ \"query\" : \"SELECT * FROM sys_journal sj WHERE sj.id = 'inv_1i7mrFgVA6eD1gnGo4q57KH9dzPZwZuBAl' ORDER BY index LIMIT 1000\" }" -o journal
```

# Development

## Format

```shell
cargo +nightly fmt
```

## Tests

```shell
cargo test test_handle_connection -- --nocapture

cargo test test_query -- --nocapture
```

