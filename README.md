# Abstract OS 

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

## Publishing

### Packages

To publish all the packages in the repo, execute the following steps:

1. Update all occurrences of the version in the Cargo.toml files
2. Publish using the following command

```bash
./publish/publish.sh
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

# Contract Migrate Ability

Migratable contracts are always a security risk. Therefore we'll outline all the migratable contracts and who's allowed
to do it here.

## Migratable

- Manager (root)
- Proxy (root)
- Add-ons (root)
- OS Factory (Abstract)
- Module Factory (Abstract)
- Version Control (Abstract)
- AnsHost

## Not Migratable

- Apis
