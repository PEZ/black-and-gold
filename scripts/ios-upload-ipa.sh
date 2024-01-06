#!/bin/bash
set -e

if [ $# -ne 2 ]; then
    echo "Usage: $0 <apple-id> <app-specific-password>"
    exit 1
fi
APPLE_ID=$1
APP_SPECIFIC_PASSWORD=$2
xcrun altool --upload-app -f Afuera.ipa -t ios -u "$APPLE_ID" -p "$APP_SPECIFIC_PASSWORD"
