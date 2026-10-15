CARGO ?= cargo
NVIM ?= nvim
CARGO_HOME ?= $(CURDIR)/build/cargo
export CARGO_HOME

.PHONY: build test check test-neovim test-deps
build:
	$(CARGO) build --release --locked

test:
	$(CARGO) test --locked

check:
	$(CARGO) fmt --all --check
	$(CARGO) clippy --all-targets --locked -- -D warnings

test-deps:
	sh scripts/fetch-test-deps.sh

test-neovim: build test-deps
	XDG_STATE_HOME="$(CURDIR)/build/neovim-state" XDG_CACHE_HOME="$(CURDIR)/build/neovim-cache" NVIM_LOG_FILE="$(CURDIR)/build/nvim.log" $(NVIM) --headless -u NONE -i NONE -l tests/conform.lua
