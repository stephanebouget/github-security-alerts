#!/bin/bash
# Setup OTA updates
cp src-tauri/tauri.conf.json.example src-tauri/tauri.conf.json
read -p "GitHub username: " USERNAME
sed -i "s/YOUR_USERNAME/$USERNAME/g" src-tauri/tauri.conf.json
echo "OTA setup complete. Run: git tag v1.1.0 && git push --tags"