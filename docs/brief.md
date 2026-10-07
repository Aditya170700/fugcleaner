# Project Brief: **Fug Cleaner** — Disk Cleaner untuk Developer

> Desktop app cross-platform (macOS, Windows, Linux) untuk menemukan dan membersihkan "sampah developer" yang memakan disk: `node_modules` lama, cache package manager, build output, Rust `target/`, cache Xcode/Docker, dan lainnya.
> Stack: **Rust + Tauri 2** (backend) × **Svelte 5** (frontend).

---

## 1. Konteks & Tujuan

**User utama:** web developer dengan laptop berdisk kecil (MacBook Air M1, RAM 8 GB, SSD 256 GB) yang disknya sering penuh.

**Masalah:** tool seperti CleanMyMac fokus ke "junk" umum, padahal pemakan disk terbesar developer adalah dependency, cache, dan build artifact yang tersebar di puluhan folder project.

**Tujuan:**
1. Scan cepat dan tunjukkan **apa yang memakan disk** per kategori, dengan ukuran yang akurat.
2. Biarkan user memilih apa yang dibersihkan, dengan **default aman**.
3. Bersihkan dengan cara yang **bisa di-undo** sebisa mungkin (pindah ke Trash).
4. App harus **ringan** (target idle < 100 MB RAM, bundle < 20 MB).

---

## 2. Tech Stack

| Bagian | Pilihan |
|---|---|
| Framework desktop | **Tauri 2.x** |
| Backend | **Rust** (stable, edition 2021) |
| Frontend | **Svelte 5** (runes: `$state`, `$derived`, `$effect`) + **SvelteKit** dengan `adapter-static` (mode SPA, `ssr = false`) |
| Bahasa frontend | **TypeScript** |
| Styling | Tailwind CSS v4 atau CSS biasa (pilih salah satu, konsisten) |
| Package manager | **pnpm** |
| Scaffold awal | `pnpm create tauri-app` → template Svelte + TypeScript |

**Crate Rust yang disarankan:**
- `jwalk` — traversal direktori paralel
- `rayon` — komputasi ukuran paralel
- `trash` — pindah ke Trash/Recycle Bin secara cross-platform
- `dirs` — path home/cache/config per OS
- `sysinfo` — total & sisa kapasitas disk
- `serde`, `serde_json` — serialisasi ke frontend
- `thiserror` — error handling
- `tokio` (sudah ada via Tauri) — task async & cancellation
- `which` — cek apakah CLI (docker, brew, xcrun) terinstal

---

## 3. Fitur MVP

1. **Dashboard disk**: kapasitas total, terpakai, sisa (bar visual).
2. **Scan kategori sistem** (cache global, lihat §4) — otomatis saat app dibuka.
3. **Scan project folder**: user memilih satu atau lebih root folder (mis. `~/Projects`, `~/Code`), app mencari artifact per project (`node_modules`, `target/`, `.next`, dll).
4. **Filter "stale"**: tandai project yang tidak aktif > N hari (default 30, bisa diubah).
5. **Daftar hasil** per kategori: ukuran, jumlah item, path, last activity. Bisa expand untuk melihat item satu per satu.
6. **Pilih & bersihkan**: checkbox per kategori dan per item, total ukuran terpilih terlihat jelas, dialog konfirmasi sebelum hapus.
7. **Progress real-time** saat scan & clean, bisa **dibatalkan**.
8. **Riwayat/log** pembersihan (berapa GB dibersihkan, kapan, item apa) disimpan lokal sebagai JSON.
9. **Settings**: root folder project, threshold stale, kategori yang di-enable, exclude list (path yang tidak boleh disentuh).

---

## 4. Kategori Pembersihan

Setiap kategori adalah satu entri data (bukan hardcode di UI). Definisikan di Rust sebagai struct, dengan path per OS.

**Kolom `method`:**
- `trash` → pindahkan ke Trash
- `delete` → hapus permanen (hanya untuk cache yang pasti bisa dibuat ulang, dan tetap butuh konfirmasi)
- `command` → jalankan CLI resmi tool tersebut (lebih aman daripada hapus file langsung)

**Kolom `risk`:** `safe` (dipilih otomatis), `caution` (tidak dipilih otomatis, ada penjelasan), `danger` (tidak dipakai di MVP).

### 4a. Cache global (scan otomatis)

| id | Nama | macOS | Linux | Windows | method | risk |
|---|---|---|---|---|---|---|
| `npm_cache` | npm cache | `~/.npm/_cacache` | `~/.npm/_cacache` | `%LOCALAPPDATA%\npm-cache\_cacache` | delete | safe |
| `pnpm_store` | pnpm store | `~/Library/pnpm/store` | `~/.local/share/pnpm/store` | `%LOCALAPPDATA%\pnpm\store` | command: `pnpm store prune` | safe |
| `yarn_cache` | Yarn cache | `~/Library/Caches/Yarn` | `~/.cache/yarn` | `%LOCALAPPDATA%\Yarn\Cache` | delete | safe |
| `bun_cache` | Bun cache | `~/.bun/install/cache` | `~/.bun/install/cache` | `%USERPROFILE%\.bun\install\cache` | delete | safe |
| `cargo_registry` | Cargo registry cache | `~/.cargo/registry/cache`, `~/.cargo/registry/src` | sama | `%USERPROFILE%\.cargo\registry\...` | delete | safe |
| `cargo_git` | Cargo git checkouts | `~/.cargo/git/checkouts` | sama | sama | delete | safe |
| `playwright` | Playwright browsers | `~/Library/Caches/ms-playwright` | `~/.cache/ms-playwright` | `%LOCALAPPDATA%\ms-playwright` | trash | caution |
| `puppeteer` | Puppeteer browsers | `~/.cache/puppeteer` | `~/.cache/puppeteer` | `%USERPROFILE%\.cache\puppeteer` | trash | caution |
| `xcode_derived` | Xcode DerivedData | `~/Library/Developer/Xcode/DerivedData` | — | — | delete | safe |
| `ios_sim` | Simulator tidak terpakai | — | — | — | command: `xcrun simctl delete unavailable` | safe |
| `xcode_archives` | Xcode Archives | `~/Library/Developer/Xcode/Archives` | — | — | trash | caution |
| `homebrew` | Homebrew cache | `$(brew --cache)` | sama | — | command: `brew cleanup --prune=all` | safe |
| `docker` | Docker (image/container/cache tak terpakai) | — | — | — | command: `docker system prune -f` (tanpa `-a` dan tanpa `--volumes`) | caution |
| `user_caches` | Cache aplikasi umum | `~/Library/Caches/*` | `~/.cache/*` | `%LOCALAPPDATA%\Temp` | trash | caution |
| `vscode_cache` | VS Code cache | `~/Library/Application Support/Code/Cache`, `CachedData`, `CachedExtensionVSIXs` | `~/.config/Code/...` | `%APPDATA%\Code\...` | delete | safe |

Catatan:
- Kategori `command` hanya muncul jika CLI-nya terinstal (`which`). Untuk Docker, cek juga daemon berjalan; jika tidak, tampilkan status "Docker tidak berjalan".
- Untuk `command`, tampilkan estimasi ukuran sebelum (jika bisa dihitung) dan hasil output CLI setelahnya.
- `user_caches`: tampilkan per sub-folder (per app) agar user bisa memilih; jangan sentuh cache milik app yang sedang berjalan jika bisa dideteksi.

### 4b. Artifact per project (scan folder pilihan user)

Sebuah folder dianggap **project** jika berisi salah satu marker: `package.json`, `Cargo.toml`, `composer.json`, `pyproject.toml`, `go.mod`.

| id | Target | Syarat | method | risk |
|---|---|---|---|---|
| `node_modules` | `node_modules/` | sejajar dengan `package.json` | trash | safe jika stale, caution jika aktif |
| `rust_target` | `target/` | sejajar dengan `Cargo.toml` | delete | safe jika stale |
| `next` | `.next/` | sejajar `package.json` | delete | safe |
| `nuxt` | `.nuxt/`, `.output/` | sejajar `package.json` | delete | safe |
| `svelte_kit` | `.svelte-kit/` | sejajar `package.json` | delete | safe |
| `turbo` | `.turbo/` | — | delete | safe |
| `parcel` | `.parcel-cache/` | — | delete | safe |
| `vite_cache` | `node_modules/.vite` | — | delete | safe |
| `build_out` | `dist/`, `build/`, `out/` | sejajar `package.json` dan **ada di `.gitignore`** project | trash | caution |
| `coverage` | `coverage/` | ada di `.gitignore` | delete | safe |
| `vendor_php` | `vendor/` | sejajar `composer.json` | trash | caution |
| `python_venv` | `.venv/`, `venv/` | sejajar `pyproject.toml`/`requirements.txt` | trash | caution |

**Definisi "last activity" project:** nilai mtime paling baru dari file di root project (tidak rekursif, abaikan folder artifact di atas) dan `.git/index` jika ada. Project dianggap **stale** jika last activity > threshold.

---

## 5. Aturan Keamanan (WAJIB, jangan dilanggar)

1. **Tidak pernah menghapus apa pun tanpa konfirmasi eksplisit user** di dialog yang menampilkan jumlah item dan total ukuran.
2. **Allowlist, bukan blocklist**: Rust hanya boleh menghapus path yang (a) dihasilkan oleh scanner di sesi yang sama, dan (b) cocok dengan definisi kategori. Command `clean` **menolak** path arbitrer dari frontend. Implementasikan dengan menyimpan hasil scan di state Rust dan frontend hanya mengirim **ID item**, bukan path.
3. **Hard-deny list** (tolak walau entah bagaimana lolos): `/`, `~`, `/System`, `/Library`, `/usr`, `/bin`, `/Applications`, `C:\Windows`, `C:\Program Files`, home user itu sendiri, `~/Documents`, `~/Desktop`, `~/Downloads`, `.git/` mana pun, dan root folder project itu sendiri.
4. **Jangan ikuti symlink** saat scan maupun hapus. Jika target adalah symlink, hapus link-nya saja, bukan isinya.
5. **Jangan menyeberang filesystem/volume** saat traversal (cek device ID), untuk menghindari masuk ke external drive atau network mount secara tidak sengaja.
6. **Validasi ulang tepat sebelum hapus**: path masih ada, masih bukan symlink, masih di dalam root yang diizinkan (canonicalize lalu cek prefix).
7. Default method untuk apa pun yang ragu adalah **`trash`**, bukan `delete`.
8. **Dry-run mode** di Settings: semua aksi hanya dicatat ke log, tidak ada file disentuh. Aktif secara default saat development (`cfg!(debug_assertions)`).
9. Jika satu item gagal dihapus (permission, file terkunci), lanjutkan ke item berikutnya dan laporkan di ringkasan akhir. Jangan panic.

---

## 6. Akurasi Ukuran

- Hitung **ukuran terpakai di disk**, bukan ukuran logis:
  - Unix: `metadata.blocks() * 512` (`std::os::unix::fs::MetadataExt`)
  - Windows: `metadata.len()` sudah cukup untuk MVP
- **Deduplikasi hardlink** dengan key `(dev, ino)` di Unix. Ini penting karena **pnpm** membuat `node_modules` berisi hardlink ke store, sehingga menghapus `node_modules` proyek pnpm membebaskan jauh lebih sedikit dari yang terlihat. Tampilkan label "pnpm (hardlinked)" dan estimasi ruang yang benar-benar dibebaskan.
- APFS clone (macOS) tidak bisa dideteksi dengan murah; cukup beri catatan kecil di UI bahwa angka adalah estimasi.
- Format ukuran pakai basis 1000 (GB) agar konsisten dengan Finder macOS.

---

## 7. Arsitektur & Struktur Folder

```
fugcleaner/
├─ docs/                     # SvelteKit frontend
│  ├─ brief.md               # Brief file
│  └─ todo.md                # Todo list
├─ src/                      # SvelteKit frontend
│  ├─ lib/
│  │  ├─ api.ts              # wrapper typed untuk invoke() & listen()
│  │  ├─ types.ts            # mirror dari struct Rust
│  │  ├─ stores/             # state global (runes, .svelte.ts)
│  │  └─ components/         # DiskBar, CategoryCard, ItemList, ConfirmDialog, ProgressBar, ...
│  └─ routes/
│     ├─ +layout.ts          # export const ssr = false; prerender = true
│     ├─ +page.svelte        # Dashboard
│     ├─ projects/+page.svelte
│     ├─ history/+page.svelte
│     └─ settings/+page.svelte
├─ src-tauri/
│  ├─ src/
│  │  ├─ main.rs
│  │  ├─ lib.rs              # setup Tauri, register commands, managed state
│  │  ├─ commands.rs         # semua #[tauri::command]
│  │  ├─ categories/         # definisi kategori per OS (mod.rs, macos.rs, linux.rs, windows.rs)
│  │  ├─ scanner/            # global.rs, projects.rs, size.rs (dedupe hardlink)
│  │  ├─ cleaner.rs          # trash/delete/command + validasi keamanan (§5)
│  │  ├─ safety.rs           # hard-deny list, canonicalize, prefix check
│  │  ├─ settings.rs         # baca/tulis settings JSON di app config dir
│  │  ├─ history.rs          # log pembersihan
│  │  └─ error.rs
│  ├─ capabilities/default.json
│  └─ tauri.conf.json
└─ package.json
```

**Plugin Tauri yang dipakai:** `tauri-plugin-dialog` (pilih folder), `tauri-plugin-opener` (buka path di Finder/Explorer & buka System Settings). Batasi permission di `capabilities/` seminimal mungkin. Frontend **tidak** diberi akses `fs` langsung; semua operasi file lewat command Rust.

---

## 8. Kontrak API (Rust ↔ Frontend)

### Types

```ts
type Risk = 'safe' | 'caution';
type Method = 'trash' | 'delete' | 'command';

interface DiskInfo { total: number; available: number; mountPoint: string }

interface Category {
  id: string; name: string; description: string;
  group: 'global' | 'project';
  method: Method; risk: Risk;
  available: boolean;          // false jika CLI tidak terinstal / path tidak ada
  unavailableReason?: string;
}

interface ScanItem {
  id: string;                  // ID internal, dipakai untuk clean (bukan path)
  categoryId: string;
  path: string;                // untuk ditampilkan saja
  bytes: number;               // ukuran di disk setelah dedupe
  projectName?: string;
  lastActivity?: number;       // unix ms
  stale?: boolean;
  note?: string;               // mis. "pnpm hardlinked"
}

interface CleanResult {
  freedBytes: number;
  succeeded: string[];         // item id
  failed: { id: string; error: string }[];
  dryRun: boolean;
}
```

### Commands

| Command | Input | Output |
|---|---|---|
| `get_disk_info` | — | `DiskInfo` |
| `list_categories` | — | `Category[]` |
| `scan_global` | `{ scanId: string }` | `ScanItem[]` (progress via event) |
| `scan_projects` | `{ scanId: string, roots: string[] }` | `ScanItem[]` |
| `cancel_scan` | `{ scanId: string }` | `void` |
| `clean_items` | `{ itemIds: string[] }` | `CleanResult` |
| `get_settings` / `save_settings` | `Settings` | `Settings` |
| `get_history` | — | `HistoryEntry[]` |
| `check_full_disk_access` | — | `boolean` (macOS saja; selain itu selalu `true`) |
| `open_in_file_manager` | `{ itemId: string }` | `void` |

### Events (Rust → Frontend)

- `scan://progress` → `{ scanId, phase, currentPath, itemsFound, bytesFound }` (throttle maks ~10x/detik)
- `clean://progress` → `{ done, total, currentPath }`

Scan berjalan di `tokio::task::spawn_blocking` agar UI tidak freeze; cancellation pakai `Arc<AtomicBool>` yang dicek di loop traversal.

---

## 9. UI/UX

**Gaya:** bersih, native-feeling, dukung light & dark mode mengikuti sistem. Font sistem (`-apple-system`, `Segoe UI`, dsb).

**Layout:** sidebar kiri (Dashboard, Projects, History, Settings) + konten utama.

**Dashboard:**
- Bar kapasitas disk di atas: terpakai / sisa / **potensi dibersihkan** (warna berbeda).
- Grid kartu kategori: ikon, nama, ukuran, jumlah item, badge risk, checkbox. Kartu kosong (0 B) dikecilkan atau disembunyikan.
- Klik kartu → panel detail berisi daftar item (path, ukuran, tombol "Show in Finder").
- Footer sticky: "**X GB dipilih**" + tombol **Clean** (disabled jika 0).

**Projects:**
- Tombol "Tambah folder" (dialog pilih folder).
- Tabel project: nama, path, last activity ("3 bulan lalu"), artifact yang ditemukan, total ukuran. Sortable per kolom.
- Toggle "Tampilkan hanya project stale". Tombol "Pilih semua yang stale".

**Konfirmasi:** dialog modal berisi ringkasan per kategori, total ukuran, dan keterangan method (Trash vs permanen). Item `delete` permanen diberi peringatan yang jelas.

**Setelah clean:** toast/ringkasan "Berhasil membebaskan 12,4 GB", daftar item gagal (jika ada), lalu refresh disk info.

**Empty & error states:** scan belum jalan, tidak ada yang bisa dibersihkan, izin kurang, CLI tidak ditemukan — semuanya punya pesan yang jelas dan aksi lanjutan.

**Bahasa UI:** Bahasa Indonesia untuk MVP, tapi simpan semua string di satu file (`src/lib/i18n/id.ts`) agar mudah ditambah English nanti.

---

## 10. Izin macOS (Full Disk Access)

- Beberapa path di `~/Library` bisa error `Operation not permitted` tanpa Full Disk Access.
- `check_full_disk_access`: coba baca path yang dilindungi (mis. `~/Library/Safari`); jika gagal karena permission → `false`.
- Jika `false`, tampilkan banner: "Beberapa cache tidak bisa dipindai. Beri Fug Cleaner izin Full Disk Access." + tombol yang membuka `x-apple.systempreferences:com.apple.preference.security?Privacy_AllFiles`.
- App tetap berfungsi tanpa izin ini; kategori yang terdampak ditandai "butuh izin".

---

## 11. Performa

- Target: scan `~/Projects` dengan ~50 project dan ~500 ribu file selesai < 15 detik di M1.
- Saat menemukan `node_modules` atau `target/`, **jangan lanjutkan traversal mencari project lain di dalamnya** (prune), tapi tetap hitung ukurannya.
- Skip folder: `.git`, `Library` (saat scan project), dan folder tersembunyi lain kecuali yang termasuk kategori.
- Batasi kedalaman scan project (default 6 level, bisa diatur).
- Frontend: virtualisasi list jika item > 200.
- Build release dengan `opt-level = "s"`, `lto = true`, `strip = true`, `codegen-units = 1` agar bundle kecil.

---

## 12. Milestone

1. **M1 – Scaffold**: Tauri 2 + SvelteKit (static) + TS + pnpm berjalan, layout sidebar, `get_disk_info` tampil di Dashboard.
2. **M2 – Scanner global**: definisi kategori macOS, `scan_global` dengan progress & cancel, ukuran akurat (blocks + dedupe hardlink).
3. **M3 – Scanner project**: pilih folder, deteksi project & artifact, last activity, filter stale.
4. **M4 – Cleaner**: `clean_items` dengan semua aturan §5, dry-run, dialog konfirmasi, ringkasan hasil.
5. **M5 – Command-based categories**: brew, docker, pnpm store, xcrun simctl.
6. **M6 – Settings, History, Full Disk Access banner.**
7. **M7 – Cross-platform**: path Linux & Windows, uji build di ketiga OS (GitHub Actions dengan `tauri-action`).
8. **M8 – Polish**: dark mode, empty states, icon app, unit test.

Kerjakan **berurutan**, dan pastikan tiap milestone bisa dijalankan (`pnpm tauri dev`) sebelum lanjut.

---

## 13. Testing

- **Unit test Rust** untuk `safety.rs`: hard-deny list, symlink, path di luar root, path traversal (`../`), canonicalize.
- **Unit test scanner** pakai folder fixture sementara (`tempfile` crate): buat project palsu dengan `node_modules`, `target/`, hardlink, symlink, lalu cek hasil scan & ukuran.
- **Test cleaner** hanya di temp dir, dan dalam dry-run untuk path nyata.
- Tidak boleh ada test yang menyentuh home directory asli developer.

---

## 14. Acceptance Criteria

- [ ] App terbuka < 2 detik, RAM idle < 100 MB, bundle macOS < 20 MB.
- [ ] Dashboard menampilkan kapasitas disk yang sama dengan Finder (selisih < 1%).
- [ ] Scan global & project menampilkan ukuran per kategori dan per item, dengan progress dan bisa dibatalkan.
- [ ] `node_modules` proyek pnpm tidak dilebih-lebihkan ukurannya (dedupe hardlink berfungsi).
- [ ] Clean hanya bisa dilakukan pada item hasil scan (frontend tidak bisa mengirim path arbitrer).
- [ ] Item dengan method `trash` benar-benar muncul di Trash dan bisa dikembalikan.
- [ ] Dry-run tidak mengubah file apa pun.
- [ ] Kegagalan per item tidak menghentikan proses dan dilaporkan.
- [ ] Banner Full Disk Access muncul jika izin belum diberikan (macOS).
- [ ] Build berhasil untuk macOS (Apple Silicon), Windows, dan Linux.

---

## 15. Di Luar Scope MVP (jangan dikerjakan dulu)

- Menghapus/uninstall aplikasi.
- Scan file besar/duplikat di seluruh disk (Documents, Downloads, dll).
- Pembersihan otomatis terjadwal / background daemon.
- Auto-update app.
- Akses root/admin atau `sudo` dalam bentuk apa pun.
- Menghapus Docker volumes atau image yang sedang dipakai.

---

## 16. Catatan untuk AI Agent

- **Keamanan file user adalah prioritas tertinggi.** Jika ragu apakah sesuatu aman dihapus, jadikan `caution` + `trash`, atau keluarkan dari scope.
- Selama development, **dry-run harus aktif**. Jangan pernah menjalankan `clean_items` non-dry-run terhadap home directory asli saat testing.
- Gunakan API terbaru: **Tauri 2** (bukan v1; perhatikan sistem `capabilities` dan plugin terpisah) dan **Svelte 5 runes** (bukan `export let` / store lama).
- Jaga struct Rust dan `types.ts` tetap sinkron (pertimbangkan `specta`/`tauri-specta` untuk generate types otomatis).
- Developer memakai laptop 256 GB: tambahkan di README cara menjaga folder `src-tauri/target` tetap kecil (`cargo clean` berkala atau `cargo install cargo-sweep`).
- Tulis README berisi cara menjalankan dev, build, dan daftar kategori beserta penjelasan risikonya.
