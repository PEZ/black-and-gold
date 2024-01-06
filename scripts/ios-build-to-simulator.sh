#!/bin/bash
set -e

BINARY_NAME="afuera"
APP_NAME="Afuera.app"

cargo build --target x86_64-apple-ios --release
cp -r assets "ios/$APP_NAME"
cp target/x86_64-apple-ios/release/$BINARY_NAME "ios/$APP_NAME/"

xcrun simctl install booted "ios/$APP_NAME"
xcrun simctl launch booted "news.afuera"