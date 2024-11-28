# Reimann

## Set the environment variables
```ini
AWS_REGION=
AWS_ACCESS_KEY_ID=
AWS_SECRET_ACCESS_KEY=
S3_BUCKET=
DA_SERVER_URL=
```

## Run the DA server
```bash
cargo run --bin cli run da
```

## Run the SMT server
```bash
cargo run --bin cli run smt
```

## Make the genesis
```bash
cargo run --bin cli genesis init
```

## Run Nexus chain
```bash
cargo run --bin cli run nexus
```

## Run the origin rollup
```bash
cargo run --bin cli run rollup --name rollup1
```

## Run the destination rollup
```bash
cargo run --bin cli run rollup --name rollup2 --port 8547 --p2p-port 30305 --authrpc-port 8553
```

## Run test transfers
```bash
cargo run --bin cli test transfers
```

## Init the full test
```bash
cargo run --bin cli test full init
```

## Run the full test
```bash
cargo run --bin cli test full run
```
