#!/bin/bash
set -e

if [ $# -ne 2 ]; then
    echo "Usage: $0 <signing-identity> <device-id>"
    exit 1
fi
SIGNING_IDENTITY=$1
DEVICE_ID=$2

BINARY_NAME="mitt-spel"
APP_NAME="MittSpel.app"

cargo build --target aarch64-apple-ios --release
cp -r assets "ios/$APP_NAME"
cp target/aarch64-apple-ios/release/$BINARY_NAME "ios/$APP_NAME/"
codesign --force --timestamp=none --sign "$SIGNING_IDENTITY" --entitlements ios/entitlements.xml "ios/$APP_NAME"
ios-deploy -d -i "$DEVICE_ID" -b "ios/$APP_NAME"