install:
    pnpm --dir frontend install

dev: install
    pnpm dlx concurrently --kill-others --names backend,frontend "cargo watch -w src -x run" "pnpm --dir frontend run dev"

build:
    cargo build --release
