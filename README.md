# nginx-fix &nbsp;![status](https://img.shields.io/badge/status-pre--release-orange?style=flat-square) ![version](https://img.shields.io/badge/version-0.0.1--alpha-blue?style=flat-square)

> Lightweight autonomous utility (~613 KB) for detecting and resolving Nginx port conflicts.

---

## ⚡ Features

- **Smart config discovery** — checks CLI args, local path, and standard Linux locations automatically
- **Friendly-fire protection** — detects a running Nginx process and leaves it untouched
- **Interactive kill** — identifies the conflicting process and prompts before issuing `SIGKILL (-9)`

---

## 📦 Installation

```bash
git clone https://github.com/your-username/nginx-fix
cd nginx-fix
cargo build --release
```

```bash
sudo cp target/release/nginx-fix /usr/local/bin/
```

---

## 🚀 Usage

```bash
sudo nginx-fix
# or with explicit config path
sudo nginx-fix /etc/nginx/nginx.conf
```

**Example output — conflict detected:**

```
nginx-fix v0.0.1-alpha

[~] Searching for nginx.conf...
[✓] Config found: /etc/nginx/nginx.conf

[~] Parsing listen directives...
[✓] Ports in use: 80, 443

[~] Checking port 80...
[!] Conflict detected on port 80

    PID   : 3847
    Binary: /usr/bin/python3
    Cmd   : python3 -m http.server 80

[?] Kill process 3847? [y/N]: y

[✓] Process 3847 terminated.
[✓] Port 80 is now free.

[~] Checking port 443...
[✓] Port 443 is free.

Done. You may now start Nginx.
```

---

## 🔧 Stack

| Component | Details |
|-----------|---------|
| Language  | Rust (stable) |
| Crates    | `listeners` |
| Binary size | ~613 KB (release) |
| Target OS | Linux |

---

## 📄 License

[MIT](LICENSE)
