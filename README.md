# Abstract 

<a href="https://codecov.io/gh/AbstractSDK/contracts" > 
 <img src="https://codecov.io/gh/AbstractSDK/contracts/branch/main/graph/badge.svg?token=FOIDUFYSCY"/> 
 </a>

<!-- ![alt text](https://github.com/AbstractSDK/contracts/blob/main/architecture.png?raw=true) -->

## Schemas

### Manual Schema Generation

To generate the schemas for all the packages in this ws, run the following command. You may have to install [cargo
workspaces(https://github.com/pksunkara/cargo-workspaces):

```bash
cargo install cargo-workspaces
```

When it is installed, run the following to generate schemas for each:

```bash
cargo ws exec --no-bail cargo schema
```

To publish the schemas to the [schema repo](https://github.com/AbstractSDK/schemas), run the following command:

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

## Documentation

The documentation is generated using [mdbook](https://rust-lang.github.io/mdBook/index.html).  
You can install mdbook and the mermaid pre-processor by running `just install-docs`.

Then you can edit the files in the `docs/src` folder and run

```shell
just serve-docs
```

This will serve you the documentation and automatically re-compiles it when you make changes.

[Release Docs](https://docs.abstract.money)
[Dev Docs](https://dev-docs.abstract.money)

## Formatting

We use `rustfmt` and [`taplo`](https://taplo.tamasfe.dev/cli/introduction.html) to format our code. To format the code, run the following command:

```bash
# format rust code
cargo fmt
# format toml files
find . -type f -iname "*.toml" -print0 | xargs -0 taplo format
```
