.PHONY: setup

## Configure local dev environment (run once after cloning).
## Sets up git hooks so cargo fmt and oxfmt run automatically on commit.
setup:
	@echo "[setup] Configuring git hooks path → .githooks"
	@git config core.hooksPath .githooks
	@echo "[setup] Done. Pre-commit hooks are active."
