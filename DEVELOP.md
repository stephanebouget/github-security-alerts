# Development Guide

## ⚡ Quick Start

```bash
git clone https://github.com/stephanebouget/github-security-alerts.git
```

```bash
cd github-security-alerts
```

```bash
npm install && npm start
```

The application should open automatically via Tauri.

## 🧰 Requirements

Make sure you have the following installed:

- Node.js >= 22
- npm >= 10
- Rust (stable)

Tauri prerequisites:
https://v2.tauri.app/start/prerequisites

Check versions:

```bash
node -v
```

```bash
npm -v
```

```bash
rustc -V
```


## 🧑‍💻 Development Mode

### Run full app (Tauri)

```bash
npm start
```

- Launches the desktop app via Tauri
- Includes backend (Rust) + frontend (Angular)

### Run frontend only (faster)

```bash
npm run web:serve
```

- Runs Angular in browser
- Hot reload enabled
- Recommended for UI development


## 🧪 Tests

Run tests:

```bash
npm test
```

> [!NOTE]
> Test coverage is currently limited. Contributions are welcome.

---

## 🐛 Debugging

- Frontend logs → Browser devtools
- Backend logs → Terminal running the app

Enable verbose logs:

```bash
RUST_LOG=debug npm start
```

## 📦 Build

### Web build

```bash
npm run web:prod
```

Output:
/dist

---

### Desktop app (Tauri)

```bash
npm run tauri:bundle
```

Output:
src-tauri/target/release/bundle/

## 📁 Project Structure

| Folder    | Description                         |
| --------- | ----------------------------------- |
| src       | Angular frontend (renderer process) |
| src-tauri | Tauri backend (Rust main process)   |


## 🔄 Development Workflow

1. Create a branch from `main`
2. Make your changes
3. Run the app locally
4. Ensure everything builds correctly
5. Submit a Pull Request

## ℹ️ Notes

- Only the `/dist` folder is included in the final bundle
- Prefer `web:serve` for frontend work to improve speed
