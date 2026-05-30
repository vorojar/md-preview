#!/usr/bin/env bash
set -euo pipefail

ROOT="$(cd "$(dirname "${BASH_SOURCE[0]}")/../.." && pwd)"
ENV_FILE="$ROOT/.env.mobile-release"
KEYSTORE_DIR="$ROOT/mobile/android/signing"
KEYSTORE="$KEYSTORE_DIR/md-preview-upload.keystore"

if [ -f "$KEYSTORE" ] || [ -f "$ENV_FILE" ]; then
  echo "[android-keystore] existing signing material found; refusing to overwrite"
  echo "[android-keystore] keystore: $KEYSTORE"
  echo "[android-keystore] env: $ENV_FILE"
  exit 0
fi

command -v keytool >/dev/null 2>&1 || {
  echo "[android-keystore] keytool missing" >&2
  exit 1
}

mkdir -p "$KEYSTORE_DIR"
store_password="$(openssl rand -base64 32 | tr -d '\n')"
key_password="$store_password"

keytool -genkeypair \
  -keystore "$KEYSTORE" \
  -alias md-preview-upload \
  -keyalg RSA \
  -keysize 4096 \
  -validity 10000 \
  -storepass "$store_password" \
  -keypass "$key_password" \
  -dname "CN=MD Preview, OU=Release, O=MD Preview, L=Local, ST=Local, C=US" >/dev/null

umask 077
cat > "$ENV_FILE" <<EOF
MD_PREVIEW_ANDROID_KEYSTORE="$KEYSTORE"
MD_PREVIEW_ANDROID_KEYSTORE_PASSWORD="$store_password"
MD_PREVIEW_ANDROID_KEY_ALIAS="md-preview-upload"
MD_PREVIEW_ANDROID_KEY_PASSWORD="$key_password"
EOF

chmod 600 "$ENV_FILE"
chmod 600 "$KEYSTORE"

echo "[android-keystore] created local ignored upload keystore"
echo "[android-keystore] keystore: $KEYSTORE"
echo "[android-keystore] env: $ENV_FILE"
