# A Simple Blockchain

```shell
cargo run -- --host <port> --port <port> --workdir <workdir> 
```

## Interact

Access via HTTP

nonce format `random-u64.random-u64` = block hash

Return hash to find nonce
```js
Method: GET
<address>:<port>/hash
```

When you find the nonce, validate the block:
```js
Method: POST
<address>:<port>/validate
Body-Type: text/plain
Body: <your-nonce>
```

To add transaction to the last unconfirmed block (unconfirmed transactions):
```js
Method: POST
<address>:<port>/send
Body-Type: json
Body: {
    sender: <string>
    recipient: <string>
    amount: <u64>
}
```