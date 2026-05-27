#!/bin/bash
set -e

cargo install --path . --force

SERVICE_NAME="sysproxyd"
SERVICE_FILE="install/sysproxyd.service"
USER_SERVICE_DIR="$HOME/.config/systemd/user"

echo "Installing $SERVICE_NAME user service..."

# Create user systemd directory if it doesn't exist
mkdir -p "$USER_SERVICE_DIR"

# Copy service file
cp "$SERVICE_FILE" "$USER_SERVICE_DIR/"

# Reload systemd daemon
systemctl --user daemon-reload

echo "Service installed. To enable and start:"
echo "  systemctl --user enable $SERVICE_NAME"
echo "  systemctl --user start $SERVICE_NAME"
echo ""
echo "To check status:"
echo "  systemctl --user status $SERVICE_NAME"
