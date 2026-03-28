#!/usr/bin/env bash
# Takes a screenshot of hyprgrid on an empty workspace
# Usage: ./assets/take-screenshot.sh

set -euo pipefail

SCRIPT_DIR="$(cd "$(dirname "$0")" && pwd)"
OUTPUT="$SCRIPT_DIR/screenshot.png"
DEMO_CONFIG="$SCRIPT_DIR/demo-config.toml"
EMPTY_WS="99"

# Remember current workspace
CURRENT_WS=$(hyprctl activeworkspace -j | jq -r '.id')

echo "Current workspace: $CURRENT_WS"
echo "Switching to empty workspace $EMPTY_WS..."

# Switch to empty workspace
hyprctl dispatch workspace "$EMPTY_WS"
sleep 0.5

# Launch hyprgrid in background
echo "Launching hyprgrid..."
hyprgrid -c demo --config "$DEMO_CONFIG" &
HYPRGRID_PID=$!
sleep 1.5

# Get hyprgrid layer-shell surface geometry
echo "Capturing hyprgrid window..."
GEOM=$(hyprctl layers -j | jq -r '
  [.. | objects | select(.namespace? == "gtk4-layer-shell" and .pid? == '"$HYPRGRID_PID"')] |
  first | "\(.x),\(.y) \(.w)x\(.h)"
')

if [ "$GEOM" != "null" ] && [ -n "$GEOM" ]; then
  echo "Window geometry: $GEOM"
  grim -g "$GEOM" "$OUTPUT"
else
  echo "Could not find hyprgrid layer, capturing full screen..."
  grim "$OUTPUT"
fi

# Kill hyprgrid
echo "Cleaning up..."
kill "$HYPRGRID_PID" 2>/dev/null || true

# Switch back
sleep 0.3
hyprctl dispatch workspace "$CURRENT_WS"

echo "Screenshot saved to: $OUTPUT"
