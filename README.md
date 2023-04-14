# Abstract 

<a href="https://codecov.io/gh/Abstract-OS/contracts" > 
 <img src="https://codecov.io/gh/Abstract-OS/contracts/branch/main/graph/badge.svg?token=FOIDUFYSCY"/> 
 </a>

<!-- ![alt text](https://github.com/Abstract-OS/contracts/blob/main/architecture.png?raw=true) -->

# Manual Schema Generation

To generate the schemas for all the packages in this ws, run the following command. You may have to install [cargo
workspaces(https://github.com/pksunkara/cargo-workspaces):

```bash
cargo install cargo-workspaces
```

When it is installed, run the following to generate schemas for each:

```bash
cargo ws exec --no-bail cargo schema
```

### Schemas

To publish the schemas to the [schema repo](https://github.com/Abstract-OS/schemas), run the following command:

```shell
cargo 
```

```bash
SCHEMA_OUT_DIR=$(cd ../schemas && echo "$PWD") \
VERSION=0.4.0 \
  cargo ws exec --no-bail bash -lc 'cargo schema && { outdir="$SCHEMA_OUT_DIR/abstract/${PWD##*/}/$VERSION"; echo $outdir; mkdir -p "$outdir"; cp -a "schema/." "$outdir"; }'
  ```

## CI

Read the [CI](./CI.md) document for more information.

## Formatting

We use `rustfmt` and [`taplo`](https://taplo.tamasfe.dev/cli/introduction.html) to format our code. To format the code, run the following command:

```bash
# format rust code
cargo fmt
# format toml files
find . -type f -iname "*.toml" -print0 | xargs -0 taplo format
```
