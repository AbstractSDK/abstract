#!/bin/bash
export PATH=$PATH:/usr/local/bin

#
# contracts pre-push hook, used to perform static analysis checks on changed files.
#
# Install the hook with the --install option.
#

project_toplevel=$(git rev-parse --show-toplevel)
git_directory=$(git rev-parse --git-dir)

install_hook() {
  mkdir -p "$git_directory/hooks"
  ln -sfv "$project_toplevel/hooks/pre-push.sh" "$git_directory/hooks/pre-push"
}

if [ "$1" = "--install" ]; then
  if [ -f "$git_directory/hooks/pre-push" ]; then
    read -r -p "There's an existing pre-push hook. Do you want to overwrite it? [y/N] " response
    case "$response" in
    [yY][eE][sS] | [yY])
      install_hook
      ;;
    *)
      printf "Skipping hook installation :("
      exit $?
      ;;
    esac
  else
    install_hook
  fi
  exit $?
fi

# cargo fmt checks
format_check() {
  printf "Starting file formatting check...\n"
  cd $project_toplevel || exit;
  
  staged_files=$(git diff --name-only --cached)
  not_staged_files=$(git diff --name-only)
  if [ -n "$staged_files" ] || [ -n "$not_staged_files" ]; then
    printf "Found staged or not-staged files. Commit or stash these first.\n"
    exit 1
  else
    printf "Running formatter...\n"
    just cargo-all fmt --all;
    find . -type f -iname "*.toml" -print0 | xargs -0 taplo format;
    # cargo workspaces exec --no-bail cargo schema >/dev/null;
    sleep 3; # Give git time to find changed files.
    git add .
    git commit -m "formatting [skip ci]"
    exit $?
  fi
}

format_check
