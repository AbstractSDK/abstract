# Treasury Dapp template

This is a template to build smart contracts for dapps that will interact with the White Whale treasury.

## Creating a new repo from template

Assuming you have a recent version of rust and cargo (v1.55.0+) installed
(via [rustup](https://rustup.rs/)),
then the following should get you a new repo to start a contract:

(Note that recent cargo-generate requires Rust 1.55 features or produces a compile error)

Install [cargo-generate](https://github.com/ashleygwilliams/cargo-generate) and cargo-run-script.
Unless you did that before, run this line now:

```sh
cargo install cargo-generate --features vendored-openssl
cargo install cargo-run-script
```

Now, use it to create your new contract.
Go to the folder in which you want to place it and run:

**Latest: 0.1.0**

```sh
cargo generate --git https://github.com/pandora-Defi-Platform/dapp-template.git --name DAPP_NAME
````

or

```sh
cargo generate --path path-to-dapp-template --name DAPP_NAME
````

You will now have a new folder called `DAPP_NAME` (I hope you changed that to something else)
containing a simple working contract using the base elements for a treasury dapp you can customize.

## Create a Repo

After generating, you have a initialized local git repo, but no commits, and no remote.
Go to a server (eg. github) and create a new upstream repo (called `YOUR-GIT-URL` below).
Then run the following:

```sh
# this is needed to create a valid Cargo.lock file (see below)
cargo check
git branch -M main
git add .
git commit -m 'Initial Commit'
git remote add origin YOUR-GIT-URL
git push -u origin main
```

# Tests
The test cases covered by this dapp are located in [the README file under src/tests/](src/tests/README.md).
