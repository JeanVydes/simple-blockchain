# A Simple Blockchain

A simple blockchain (learning purporses).

### Run

Default values 

```ts
pub const NODE_DEFAULT_ADDRESS = "127.0.0.1";
pub const NODE_DEFAULT_PORT: u16 = 5954;
pub const NODE_DEFAULT_DIR_DATA: &str = "/tmp/blockchain";
pub const BLOCK_REWARD: u64 = 100;
```

Run Default

```shell
cargo run --debug
```

Run Custom

```shell
cargo run -- --host <port> --port <port> --workdir <workdir> --debug
```

### Interact

Access via HTTP

nonce format `random-u64.random-u64` = block hash in sha256

Return hash to find nonce
```js
Method: GET
Endpoint: /hash
```

When you find the nonce, validate the block:
```js
Method: POST
Endpoint: /validate
Body-Type: text/plain
Body: <your-nonce>
```

To add transaction to the last unconfirmed block (unconfirmed transactions):
```js
Method: POST
Endpoint: /send
Body-Type: json
Body: {
    sender: <string>
    recipient: <string>
    amount: <u64>
}
```

Get block data:
```js
Method: GET
Endpoint: /get/block?id=<block-id>
```

Get last block data:
```js
Method: GET
Endpoint: /get/lastblock
```

Get unconfirmed transactions:
```js
Method: GET
Endpoint: /get/unconfirmedtransactions
```