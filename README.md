# GitHub Security Alerts

A modern desktop application that monitors security vulnerabilities across your GitHub repositories in real-time. Built with Angular and Rust/Tauri for a lightweight, performant experience.

<img width="829" height="414" alt="image" src="https://github.com/user-attachments/assets/9e17b4a1-154e-46c8-a2b8-9abe6f33d9c1" />

<img width="829" height="102" alt="image" src="https://github.com/user-attachments/assets/69e1730a-720b-4647-9025-6c776309b75e" />

## 🛡️ Installation Security Notice

When installing this application, you may encounter security warnings from your operating system.

Why this happens: **This application is not code-signed** with a commercial certificate, which is a costly process for open-source projects.

## 📋 Features

### Core Functionality

- **Real-time Security Monitoring**: Track Dependabot security alerts across all your repositories
- **Repository Management**: Select and manage which repositories to monitor
- **GitHub Integration**: Seamless authentication via GitHub personal access tokens
- **System Tray Integration**: Minimize to system tray with icon status indicators
- **Auto-refresh**: Automatic alert updates every hour (configurable)

## 📥 Download

Get the latest version of GitHub Security Alerts:

- **[Download Latest Release](https://github.com/stephanebouget/github-security-alerts/releases/latest)**

Available for Windows, macOS, and Linux.

## ⚠️ Prerequisites

**GitHub Advanced Security Features**

To ensure this application works correctly, you must enable GitHub Advanced Security features on the repositories you want to monitor.:

- Dependabot alerts must be activated
  <img width="767" height="88" alt="image" src="https://github.com/user-attachments/assets/5517e326-cedc-47c9-99b9-e532b0c25959" />
- Security advisories should be enabled
- For private repositories, you may need a GitHub Enterprise or GitHub Advanced Security license

## 🎯 Usage Guide

### First Launch

1. **Authenticate**

<div align="center">
   <img width="416" height="596" alt="image" src="https://github.com/user-attachments/assets/8c54de72-d39b-430f-a49d-9af2383e135d" />
</div>

- Paste your GitHub personal access token in the login form
- The app validates the token and saves it securely

2. **Select Repositories**

<div align="center">
   <img width="416" height="596" alt="image" src="https://github.com/user-attachments/assets/bb011e76-d2e9-4db1-9aff-cbd508fbfe10" />
</div>

- Click the "📦 Repositories" button in the header
- Expand owners (users/organizations) to see their repositories
- Select repositories you want to monitor
- Click "✓ Done - View Alerts"

3. **View Alerts**

<div align="center">
   <img width="416" height="596" alt="image" src="https://github.com/user-attachments/assets/f5948b16-08e5-4e4c-8131-3ce77da7f79b" />
</div>

- The main "Alerts" view shows a summary of total security alerts
- Each repository displays its alert count
- Click any repository to open it on GitHub in your browser
- Checkmark (✓) indicates no alerts, numbers show active alerts

## ✨ Additional Features

- **OTA Updates**: Automatic over-the-air updates with signature verification for security

  <img width="397" height="225" alt="image" src="https://github.com/user-attachments/assets/3a9e60af-c479-48fc-8ddd-c7c311d567ac" />

- **API Rate Limits Usage**: Real-time monitoring of GitHub API rate limits and usage statistics

  <img width="394" height="252" alt="image" src="https://github.com/user-attachments/assets/8359f933-5517-411e-b549-d50ef4a42318" />

- **Auto-refresh Interval Customizable**: Configure refresh intervals from 1 minute to 4 hours based on your needs

  <img width="404" height="97" alt="image" src="https://github.com/user-attachments/assets/9ce75e85-e589-4013-addf-355acc09bd10" />


## 📝 License

This project is licensed under the MIT License - see the LICENSE file for details.
