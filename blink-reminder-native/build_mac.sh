#!/bin/bash
set -e
# rm ~/.blink-reminder/config.json

echo "Building release binary..."
CARGO_TARGET_DIR=./target
cargo bundle --release

APP_PATH="./target/release/bundle/osx/Blink Reminder.app"
PLIST_PATH="$APP_PATH/Contents/Info.plist"
DMG_PATH="./target/release/bundle/dmg/Blink Reminder.dmg"

echo "Injecting LSUIElement into Info.plist..."
# Use PlistBuddy to ensure the plist is correctly modified
/usr/libexec/PlistBuddy -c "Add :LSUIElement bool true" "$PLIST_PATH" || /usr/libexec/PlistBuddy -c "Set :LSUIElement true" "$PLIST_PATH"

echo "Re-signing the app bundle..."
codesign --force --deep --sign - "$APP_PATH"

echo "Recreating DMG..."
rm -f "$DMG_PATH"
hdiutil create -volname "Blink Reminder" -srcfolder "$APP_PATH" -ov -format UDZO "$DMG_PATH"

echo "Done! DMG is ready at $DMG_PATH"
