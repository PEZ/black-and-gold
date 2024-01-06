#!/bin/bash
set -e

if [ $# -ne 1 ]; then
    echo "Usage: $0 <dev | dist>"
    exit 1
fi
PROFILE_TYPE=$1

APP_NAME="Afuera"
APP_FOLDER="ios/$APP_NAME.app"

mkdir -p "$APP_FOLDER"
cp ios/Info.plist "$APP_FOLDER/"
cp ios/entitlements.xml "$APP_FOLDER/"
cp ios/app-store-assets/*.png "$APP_FOLDER/"
cp "ios/$PROFILE_TYPE.mobileprovision" "$APP_FOLDER/embedded.mobileprovision"
