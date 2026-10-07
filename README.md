# Fug Cleaner 🧹

> **Disk Cleaner Cerdas, Aman, dan Ringan untuk Developer**  
> Dibuat secara native dengan **Tauri 2.x (Rust)** dan **Svelte 5 (TypeScript)**.

![Fug Cleaner Banner](docs/logo/cover.png)

---

## 🌟 Fitur Utama

- **🚀 Pemindaian Cepat & Paralel**: Menggunakan `jwalk` dan `rayon` di backend Rust untuk pemindaian multi-threaded berkecepatan tinggi tanpa memblokir UI.
- **🛡️ Sistem Keamanan Berlapis (Safety Module)**:
  - **Hard-Deny List**: Melindungi direktori sistem (`/`, `/System`, `/usr`, `C:\Windows`, dll), home directory, folder dokumen pribadi, dan direktori `.git/`.
  - **Symlink Protection**: Memeriksa `symlink_metadata` dan menolak operasi terhadap symlink untuk mencegah penghapusan target yang tidak diinginkan.
  - **Cross-Filesystem Protection**: Mencegah traversal melompat ke volume atau external drive lain.
  - **Double Validation**: Validasi ulang path tepat sebelum eksekusi pembersihan.
  - **Dry-Run Simulation**: Mode simulasi aktif default selama development untuk memeriksa apa yang akan dihapus tanpa menyentuh disk.
- **📦 Deteksi Cerdas Project & Artifact**:
  - Mendeteksi project berbasis marker (`package.json`, `Cargo.toml`, `composer.json`, `pyproject.toml`, `go.mod`).
  - Menghitung **Last Activity** & menandai project yang sudah **stale** (> 30 hari).
  - **Deduplikasi Hardlink (pnpm)**: Menghitung ukuran sesungguhnya di disk (`blocks * 512` dan tracking inode `(dev, ino)`).
- **⚙️ Kategori Berbasis CLI**: Menjalankan perintah resmi seperti `pnpm store prune`, `brew cleanup`, `xcrun simctl delete unavailable`, dan `docker system prune -f` dengan timeout dan visualisasi output terminal.
- **📊 Riwayat & Pengaturan Persisten**: Log pembersihan tersimpan lokal dalam format JSON dengan ringkasan total ruang kumulatif yang dibebaskan.
- **🔒 Integrasi Full Disk Access (macOS)**: Deteksi izin otomatis dan tautan langsung ke preferensi sistem macOS.

---

## 📂 Kategori Pembersihan & Tingkat Risiko

| ID Kategori | Nama Kategori | OS | Metode | Tingkat Risiko | Keterangan |
|---|---|---|---|---|---|
| `npm_cache` | npm cache | macOS, Linux, Windows | Delete | `Safe` | Cache global package tarball npm |
| `pnpm_store` | pnpm store | macOS, Linux, Windows | Command (`pnpm store prune`) | `Safe` | Paket unreferenced di store global pnpm |
| `yarn_cache` | Yarn cache | macOS, Linux, Windows | Delete | `Safe` | Cache paket Yarn v1/v2 |
| `bun_cache` | Bun cache | macOS, Linux, Windows | Delete | `Safe` | Cache download package Bun |
| `cargo_registry` | Cargo registry | macOS, Linux, Windows | Delete | `Safe` | Cache crates.io registry & git index |
| `cargo_git` | Cargo git checkouts | macOS, Linux, Windows | Delete | `Safe` | Cache repository git dependencies Cargo |
| `playwright` | Playwright browsers | macOS, Linux, Windows | Trash | `Caution` | Binary headless browser Playwright |
| `puppeteer` | Puppeteer browsers | macOS, Linux, Windows | Trash | `Caution` | Binary browser Puppeteer |
| `xcode_derived` | Xcode DerivedData | macOS | Delete | `Safe` | Build cache & index Xcode |
| `ios_sim` | Simulator tidak terpakai | macOS | Command (`xcrun simctl delete unavailable`) | `Safe` | Cache runtime simulator tidak terpakai |
| `xcode_archives` | Xcode Archives | macOS | Trash | `Caution` | Arsip build aplikasi lama Xcode |
| `homebrew` | Homebrew cache | macOS, Linux | Command (`brew cleanup --prune=all`) | `Safe` | Cache botol dan source unduhan Homebrew |
| `docker` | Docker system prune | macOS, Linux, Windows | Command (`docker system prune -f`) | `Caution` | Image & container menggantung (tanpa `-a` / `--volumes`) |
| `vscode_cache` | VS Code cache | macOS, Linux, Windows | Delete | `Safe` | Cache data dan ekstensi VSIX VS Code |
| `user_caches` | Cache aplikasi umum | macOS, Linux, Windows | Trash | `Caution` | Cache per-subfolder di folder Caches/Temp pengguna |
| `node_modules` | Project node_modules | Semua | Trash | `Safe` (Stale) / `Caution` (Aktif) | Folder dependencies JavaScript/TypeScript |
| `rust_target` | Project target folder | Semua | Delete | `Safe` (Stale) / `Caution` (Aktif) | Build artifact compiler Rust Cargo |
| `next` / `nuxt` / `svelte_kit` | Framework build caches | Semua | Delete | `Safe` | Artifact compiler Next.js, Nuxt, SvelteKit |

---

## 🚀 Prasyarat & Instalasi

### Prasyarat:
- **Rust** (versi stable terbaru): `curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh`
- **Node.js** (≥ 20.x) & **pnpm**: `corepack enable && corepack prepare pnpm@latest --activate`
- **Xcode Command Line Tools** (untuk macOS): `xcode-select --install`
- **Linux Packages** (untuk Ubuntu/Debian): `sudo apt install libwebkit2gtk-4.1-dev build-essential curl wget file libxdo-dev libssl-dev libayatana-appindicator3-dev librsvg2-dev`

### Menjalankan Mode Development:

```bash
# 1. Clone repository
git clone https://github.com/Aditya170700/fugcleaner.git
cd fugcleaner

# 2. Install dependencies frontend
pnpm install

# 3. Jalankan aplikasi dalam mode dev (dry-run aktif secara default)
pnpm tauri dev
```

### Melakukan Build Production:

```bash
pnpm tauri build
```

Hasil binary/bundle akan tersedia di direktori `src-tauri/target/release/bundle/`.

---

## 🧪 Pengujian & Validasi Kode

```bash
# Frontend type check & test build
pnpm check
pnpm build

# Rust unit tests
cd src-tauri
cargo test

# Rust Clippy linter
cargo clippy -- -D warnings
```

---

## ⚡ Metrik Performa & Ukuran

Berdasarkan hasil profil release dan pengujian:
- **Waktu Buka Aplikasi (Cold Startup)**: `< 1.2 detik`
- **Penggunaan Memori (RAM Idle)**: `~65 MB`
- **Ukuran Bundle Aplikasi (macOS DMG/App)**: `< 15 MB` (berkat optimasi profile release: `opt-level = "s"`, `lto = true`, `strip = true`, `codegen-units = 1`)
- **Akurasi Kapasitas Disk**: Selisih `< 0.5%` terhadap macOS Finder (menggunakan format base-1000 desimal standar).

---

## 💡 Tips Menjaga Direktori Build Tetap Ramping

Proyek Rust dapat menghasilkan file cache `target/` yang besar selama development. Gunakan perintah berikut secara berkala:

```bash
# Bersihkan seluruh artifact build lokal
cd src-tauri && cargo clean

# Atau bersihkan build dependencies lama dengan cargo-sweep (jika terpasang)
cargo sweep -time 30
```

---

## 📄 Lisensi

Distributed under the **MIT License**.
