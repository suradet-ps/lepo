# Lepo — GitHub Repo Monitor
#
# Convenience targets so the app can be built/served from the workspace ROOT,
# even though Trunk requires index.html next to the `app` crate (crates/app).
# Trunk 0.21 has no `-C` flag, so each target changes into crates/app first.

APP_DIR := crates/app

.PHONY: serve build release check clippy clippy-wasm test fmt fmt-check clean

## serve: start the Trunk dev server with hot reload (from root)
serve:
	cd $(APP_DIR) && trunk serve

## build: build the wasm bundle into crates/app/dist
build:
	cd $(APP_DIR) && trunk build

## release: optimized production build
release:
	cd $(APP_DIR) && trunk build --release

## check: type-check the app for wasm
check:
	cargo check -p app --target wasm32-unknown-unknown

## clippy: lint everything (native)
clippy:
	cargo clippy --workspace -- -D warnings

## clippy-wasm: lint everything for wasm
clippy-wasm:
	cargo clippy --workspace --target wasm32-unknown-unknown -- -D warnings

## test: run offline unit tests (models + github-api)
test:
	cargo test --workspace --exclude app

## fmt: format all crates
fmt:
	cargo fmt --all

## fmt-check: verify formatting
fmt-check:
	cargo fmt --all --check

## clean: remove build artifacts
clean:
	cargo clean
	rm -rf $(APP_DIR)/dist
