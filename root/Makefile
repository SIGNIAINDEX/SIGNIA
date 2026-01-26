SHELL := /usr/bin/env bash
.ONESHELL:
.SHELLFLAGS := -euo pipefail -c

.PHONY: bootstrap build lint test fmt clean docker-up docker-down solana-local anchor-build

bootstrap:
	./scripts/bootstrap.sh

build:
	./scripts/build_all.sh

lint:
	./scripts/lint_all.sh

test:
	./scripts/test_all.sh

fmt:
	cargo fmt --all
	pnpm -r format || true

clean:
	rm -rf ./.data
	cargo clean || true
	pnpm -r clean || true

docker-up:
	docker compose up -d --build

docker-down:
	docker compose down -v

solana-local:
	./scripts/local_solana.sh

anchor-build:
	./scripts/anchor_build.sh
