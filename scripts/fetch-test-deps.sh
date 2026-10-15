#!/bin/sh
set -eu
project_dir=$(CDPATH= cd -- "$(dirname -- "$0")/.." && pwd)
dependency_dir="$project_dir/build/test-deps/conform.nvim"
revision=016802de402556da54c36bd7359b441266b01cdd
mkdir -p "$dependency_dir"
if [ ! -d "$dependency_dir/.git" ]; then
  git -C "$dependency_dir" init -q
fi
if ! git -C "$dependency_dir" cat-file -e "$revision^{commit}" 2>/dev/null; then
  git -C "$dependency_dir" fetch --depth 1 https://github.com/stevearc/conform.nvim.git "$revision"
fi
git -C "$dependency_dir" checkout --detach "$revision"
