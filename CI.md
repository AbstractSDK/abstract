# Setup CI

We use a separate [rust-ci]() repository for CI jobs. 

## Add the remote rust-ci repo

```shell
git remote add ci https://github.com/Abstract-OS/rust-ci.git
```

Then fetch the remote repo:

```shell
git fetch ci
```

and to pull new changes:

```shell
git merge ci/main --allow-unrelated-histories         
```

If you're doing this for the first time you need to merge this branch without squashing to keep the commit history of the ci repo. Otherwise new merges will create conflicts.