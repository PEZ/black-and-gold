#!/bin/bash
set -e
SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"

if [ $# -ne 1 ]; then
    echo "Usage: $0 <signing-identity>"
    exit 1
fi
SIGNING_IDENTITY=$1

BINARY_NAME="afuera"
APP_NAME="Afuera"

"$SCRIPT_DIR/ios-init-bundle.sh" dist

cargo build --target aarch64-apple-ios --release
cp -r assets "ios/$APP_NAME.app"
cp target/aarch64-apple-ios/release/$BINARY_NAME "ios/$APP_NAME.app/"
codesign --force --timestamp=none --sign "$SIGNING_IDENTITY" --entitlements ios/entitlements.xml "ios/$APP_NAME.app"
mv "ios/$APP_NAME.app" "Payload"
zip -r "ios/$APP_NAME.ipa" "Payload"
mv "Payload" "ios/$APP_NAME.app"