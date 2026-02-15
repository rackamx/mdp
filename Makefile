.PHONY: test coverage coverage-open coverage-gate asan-regression

test:
	cargo test

coverage:
	cargo llvm-cov --workspace --all-features --summary-only

coverage-open:
	cargo llvm-cov --workspace --all-features --open

coverage-gate:
	cargo llvm-cov --workspace --all-features --summary-only \
		--fail-under-regions 90 \
		--fail-under-functions 90 \
		--fail-under-lines 90

asan-regression:
	RUSTFLAGS="-Z sanitizer=address" \
	ASAN_OPTIONS="detect_leaks=1:strict_string_checks=1" \
	cargo +nightly test --tests --target x86_64-unknown-linux-gnu
