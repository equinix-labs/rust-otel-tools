MAKEFLAGS += --no-builtin-rules
.SUFFIXES:
#.NOTPARALLEL:
.PHONY: all build clean demo docs help release test

all: help

README.md: src/lib.rs ## Regenerate README.md
	cargo rdme -r README.md
	git diff --exit-code

docs: README.md ## Prepare external docs
	cargo doc --no-deps --all-features

help: Makefile ## Print help
	@grep -h "##" $(MAKEFILE_LIST) | grep -v grep | sed -e 's/:.*##/#/' | column -c 2 -t -s#

lint: ## Lint with Cargo
	cargo check

test: ## Test with Cargo
	cargo test
	cargo test --doc
	@echo Ensuring that docs build
	cargo doc --workspace --no-deps --all-features
