# signia-cli

`signia-cli` is the command line interface for SIGNIA.

It supports:
- `signia compile <input>`: compile a structure payload into deterministic artifacts (schema/manifest/proof)
- `signia verify --root <hex> --leaf <hex> --proof <json>`: verify a Merkle inclusion proof
- `signia fetch <object-id>`: retrieve an artifact from the local store
- `signia plugins`: list supported plugins
- `signia doctor`: environment checks
- `signia publish`: placeholder for on-chain registry publish wiring

## Install (workspace)

```bash
cargo build -p signia-cli
./target/debug/signia --help
```

## Examples

Compile a JSON file:

```bash
signia compile ./examples/repo.json --out ./out
```

Compile from a URL:

```bash
signia compile https://example.com/openapi.json --kind openapi --out ./out
```

Verify a proof:

```bash
signia verify --root <hex> --leaf <hex> --proof ./out/proof.json
```

Fetch an object:

```bash
signia fetch <object-id> --to ./artifact.bin
```

## Output

By default, output is human readable.
Use `--json` to emit machine-readable JSON on stdout.

## License

MIT OR Apache-2.0
