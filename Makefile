.PHONY: check format test lint

check:
	@cargo check

format:
	@cargo fmt

test:
	@cargo test

lint:
	@cargo clippy

checks: check format test lint
	@git status
	@echo All checks passed!
