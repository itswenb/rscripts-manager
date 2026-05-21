#!/usr/bin/env bash
#
# 1. 如果 .env 不存在，自动生成
# 2. ADMIN_USER=admin
# 3. ADMIN_PASSWORD=admin
# 4. SECRET 自动随机生成
# 5. 执行 cargo build --release
# 6. 启动 ./target/release/ripeline
#
# 也支持临时指定端口：
#
# PORT=9100 ./run-ripeline.sh

set -euo pipefail

ROOT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
cd "$ROOT_DIR"

random_secret() {
  if command -v openssl >/dev/null 2>&1; then
    openssl rand -hex 32
  else
    LC_ALL=C tr -dc 'A-Za-z0-9' </dev/urandom | head -c 64
  fi
}

if [ ! -f ".env" ]; then
  SECRET="$(random_secret)"
  cat > .env <<EOF
DATABASE_URL=sqlite:ripeline.db?mode=rwc
PORT=${PORT:-9000}
DATA_DIR=~/.ripeline
SECRET=${SECRET}
ADMIN_USER=admin
ADMIN_PASSWORD=admin
EOF
  echo "Created .env with ADMIN_USER=admin and ADMIN_PASSWORD=admin"
fi

cargo build --release

exec ./target/release/ripeline
