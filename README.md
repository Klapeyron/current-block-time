# current-block-time

Returns time of latest Ethereum block exposed via "/currentBlockTime" REST API

## Usage
```
cargo run --release
```

For test query
```
curl localhost:2137/currentBlockTime
```

## Docker

```bash
docker build -t current-block-time .
docker run -p 2137:2137 current-block-time
```