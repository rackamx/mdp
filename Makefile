.PHONY: test coverage coverage-open

test:
	cargo test

coverage:
	cargo llvm-cov --workspace --all-features --summary-only

coverage-open:
	cargo llvm-cov --workspace --all-features --open
