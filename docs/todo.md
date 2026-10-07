# TODO — Fug Cleaner (Disk Cleaner untuk Developer)

> Panduan pengerjaan langkah demi langkah. Acuan utama: **`brief.md`** (nomor § di bawah merujuk ke bagian brief).

## Cara Pakai File Ini (untuk AI Agent)

1. Kerjakan **satu milestone dalam satu waktu**, berurutan dari atas. Jangan lompat ke milestone berikutnya sebelum semua item ✅ dan bagian **Verifikasi** lolos.
2. Centang item (`- [x]`) segera setelah selesai, dan perbarui file ini di setiap commit.
3. Jika menemukan pekerjaan tambahan, tambahkan sebagai item baru di milestone yang relevan, jangan dikerjakan diam-diam.
4. Jika ada keputusan yang tidak tercakup di brief, catat di bagian **Catatan & Keputusan** di bawah, lalu tanya user jika menyangkut penghapusan file.
5. **Dry-run wajib aktif** selama development. Jangan pernah menjalankan pembersihan nyata di home directory asli.
6. Commit kecil dan sering, dengan pesan yang jelas (mis. `feat(scanner): dedupe hardlink by inode`).

---

## M0 — Persiapan

- [x] Cek versi tool: `rustc --version` (stable terbaru), `node -v` (≥ 20), `pnpm -v`
- [x] Pastikan prasyarat Tauri 2 untuk macOS terpasang (Xcode Command Line Tools)
- [x] Inisialisasi git repo + `.gitignore` (Rust, Node, `.svelte-kit`, `src-tauri/target`)
- [x] Buat `CHANGELOG.md` kosong

**Verifikasi:** semua command di atas berjalan tanpa error.

---

## M1 — Scaffold Project

- [x] `pnpm create tauri-app` → nama `fugcleaner`, template **Svelte + TypeScript**, package manager **pnpm**
- [x] Pastikan memakai **Tauri 2.x** dan **Svelte 5**
- [x] Konfigurasi SvelteKit `adapter-static`, `src/routes/+layout.ts`: `export const ssr = false; export const prerender = true;`
- [x] Set up styling (CSS biasa dengan design tokens & light/dark mode sistem)
- [x] Buat struktur folder Rust sesuai §7: `commands.rs`, `categories/`, `scanner/`, `cleaner.rs`, `safety.rs`, `settings.rs`, `history.rs`, `error.rs`
- [x] Buat `error.rs` dengan `thiserror`, error bisa di-serialize ke frontend
- [x] Tambahkan profile release di `Cargo.toml`: `opt-level = "s"`, `lto = true`, `strip = true`, `codegen-units = 1`
- [x] Layout dasar: sidebar (Dashboard, Projects, History, Settings) + area konten, routing ke 4 halaman
- [x] `src/lib/i18n/id.ts` untuk semua string UI
- [x] Command `get_disk_info` pakai `sysinfo` → tampilkan **DiskBar** di Dashboard (total, terpakai, sisa)
- [x] `src/lib/api.ts` (wrapper typed untuk `invoke`/`listen`) dan `src/lib/types.ts`
- [x] Pertimbangkan `tauri-specta` untuk generate types otomatis; catat keputusan di bawah

**Verifikasi:**
- [x] `pnpm tauri dev` / build jalan, sidebar berfungsi, DiskBar menampilkan angka yang mendekati Finder (selisih < 1%)
- [x] `cargo clippy` tanpa warning

---

## M2 — Safety Module & Scanner Global

> Kerjakan `safety.rs` **lebih dulu** sebelum scanner apa pun.

### Safety (§5)
- [x] Hard-deny list (§5.3) untuk macOS, Linux, Windows
- [x] Fungsi `is_path_allowed(path, allowed_roots)` : canonicalize → cek tidak di deny list → cek prefix root → tolak symlink
- [x] Deteksi perpindahan filesystem/volume (bandingkan device ID)
- [x] Unit test: deny list, symlink, `../` traversal, path di luar root, home dir, `.git/`

### Definisi kategori (§4a)
- [x] Struct `CategoryDef { id, name, description, group, method, risk, paths_by_os, command }`
- [x] Isi kategori global untuk **macOS** dulu (Linux & Windows di M7)
- [x] Command `list_categories` → set `available` + `unavailableReason` (path tidak ada / CLI tidak terinstal via `which`)

### Perhitungan ukuran (§6)
- [x] `scanner/size.rs`: ukuran di disk = `blocks() * 512` (Unix), `len()` (Windows)
- [x] Dedupe hardlink pakai `HashSet<(dev, ino)>`
- [x] Jangan ikuti symlink
- [x] Unit test dengan `tempfile`: file biasa, hardlink (dihitung sekali), symlink (tidak diikuti)

### Scanner global
- [x] `scan_global({ scanId })` di `spawn_blocking`, traversal paralel (`jwalk` + `rayon`)
- [x] Simpan hasil scan di **managed state** Rust (`Mutex<HashMap<ItemId, ScanItem + real path>>`)
- [x] Event `scan://progress` (throttle maks ~10x/detik)
- [x] `cancel_scan` via `Arc<AtomicBool>`
- [x] `user_caches`: pecah per sub-folder (per aplikasi)

### UI Dashboard
- [x] Scan otomatis saat app dibuka + tombol "Scan ulang"
- [x] **ProgressBar** + tombol batal
- [x] Grid **CategoryCard** (ikon, nama, ukuran, jumlah item, badge risk, checkbox); `safe` tercentang default, `caution` tidak
- [x] Kategori 0 B dikecilkan/disembunyikan; kategori `available: false` tampil abu-abu dengan alasan
- [x] Panel detail **ItemList** per kategori + tombol "Show in Finder" (`open_in_file_manager`)
- [x] DiskBar menampilkan segmen "potensi dibersihkan"
- [x] Footer sticky: "X GB dipilih" + tombol Clean (masih disabled / belum terhubung)

**Verifikasi:**
- [x] Scan selesai, angka per kategori masuk akal dibanding `du -sh` di terminal
- [x] Cancel menghentikan scan dalam < 1 detik
- [x] UI tidak freeze selama scan
- [x] Semua unit test lulus

---

## M3 — Scanner Project

- [x] Plugin `tauri-plugin-dialog` untuk memilih folder; simpan root folder di settings
- [x] Deteksi project berdasarkan marker (§4b): `package.json`, `Cargo.toml`, `composer.json`, `pyproject.toml`, `go.mod`
- [x] Deteksi artifact per project sesuai tabel §4b, termasuk syarat sejajar marker
- [x] `build_out` & `coverage`: hanya jika tercantum di `.gitignore` project (parse sederhana sudah cukup)
- [x] **Prune traversal** di `node_modules`, `target/`, `.git` (§11); tetap hitung ukurannya
- [x] Batas kedalaman scan (default 6)
- [x] Hitung **last activity** (§4b) dan flag **stale** (threshold default 30 hari)
- [x] Deteksi project pnpm (`pnpm-lock.yaml` atau hardlink) → `note: "pnpm (hardlinked)"` + ukuran yang benar-benar dibebaskan
- [x] Risk dinamis: `node_modules`/`target` = `safe` jika stale, `caution` jika aktif
- [x] Halaman **Projects**: tombol tambah/hapus root folder, tabel (nama, path, last activity relatif, artifact, total ukuran), sorting per kolom
- [x] Toggle "Hanya project stale" + tombol "Pilih semua yang stale"
- [x] Virtualisasi list / paginasi jika item banyak

**Verifikasi:**
- [x] Buat folder fixture / unit test berisi project palsu (Node, Rust, pnpm) → hasil scan sesuai harapan
- [x] Scan `~/Projects` nyata: tidak ada project terlewat atau terhitung ganda
- [x] Unit test untuk deteksi project, last activity, dan pruning lulus

---

## M4 — Cleaner (paling kritis)

- [ ] `clean_items({ itemIds })`: **hanya menerima ID**, ambil path dari state Rust hasil scan
- [ ] Validasi ulang setiap item tepat sebelum hapus (`safety::is_path_allowed`, masih ada, bukan symlink)
- [ ] Method `trash` → crate `trash`; method `delete` → `remove_dir_all` (setelah validasi)
- [ ] Gagal per item → lanjut ke item berikutnya, catat di `failed`
- [ ] **Dry-run**: aktif default saat `cfg!(debug_assertions)`, juga bisa di-toggle di Settings; hanya log, tidak menyentuh file
- [ ] Event `clean://progress`
- [ ] Hapus item yang sudah dibersihkan dari state, lalu refresh `get_disk_info`
- [ ] **ConfirmDialog**: ringkasan per kategori, total ukuran, method (Trash vs permanen), peringatan jelas untuk `delete`
- [ ] Ringkasan hasil: "Berhasil membebaskan X GB" + daftar item gagal
- [ ] Badge "DRY RUN" yang jelas di UI saat mode aktif
- [ ] Unit test cleaner **hanya di temp dir**: ID tidak dikenal ditolak, path di deny list ditolak, symlink tidak diikuti, kegagalan parsial ditangani

**Verifikasi:**
- [ ] Dry-run di data nyata: tidak ada file yang berubah (cek ukuran folder sebelum/sesudah)
- [ ] Non-dry-run **hanya** di folder fixture: item `trash` muncul di Trash dan bisa dikembalikan
- [ ] Coba kirim ID palsu / path dari devtools → ditolak

---

## M5 — Kategori Berbasis Command

- [ ] Modul runner command: jalankan CLI via `std::process::Command`, timeout, tangkap stdout/stderr
- [ ] `pnpm store prune`
- [ ] `brew cleanup --prune=all` (path cache dari `brew --cache`)
- [ ] `xcrun simctl delete unavailable`
- [ ] `docker system prune -f` (**tanpa** `-a` dan `--volumes`); cek daemon berjalan dulu
- [ ] Estimasi ukuran sebelum (jika bisa) dan ukuran yang dibebaskan setelahnya (ukur ulang path)
- [ ] Tampilkan output CLI di panel hasil (bisa di-expand)
- [ ] Hormati dry-run: tampilkan command yang *akan* dijalankan tanpa mengeksekusinya

**Verifikasi:**
- [ ] Setiap kategori command tersembunyi/nonaktif jika CLI tidak ada
- [ ] Docker mati → pesan "Docker tidak berjalan", tidak crash

---

## M6 — Settings, History, Full Disk Access

### Settings
- [ ] `settings.rs`: simpan JSON di app config dir (`get_settings` / `save_settings`)
- [ ] Field: root folder project, threshold stale, kategori aktif, exclude list, kedalaman scan, dry-run
- [ ] Exclude list ikut divalidasi di scanner **dan** cleaner
- [ ] Halaman Settings dengan form + validasi

### History
- [ ] `history.rs`: append entri setiap clean (waktu, total dibebaskan, item, dry-run atau tidak)
- [ ] Halaman History: daftar entri + total kumulatif yang sudah dibersihkan

### Full Disk Access (macOS, §10)
- [ ] `check_full_disk_access`: coba baca `~/Library/Safari`
- [ ] Banner + tombol buka System Settings (`x-apple.systempreferences:com.apple.preference.security?Privacy_AllFiles`) via `tauri-plugin-opener`
- [ ] Kategori yang gagal dipindai karena izin ditandai "butuh izin"

**Verifikasi:**
- [ ] Settings tersimpan setelah app ditutup & dibuka lagi
- [ ] Banner muncul/hilang sesuai status izin

---

## M7 — Cross-Platform

- [ ] Lengkapi path kategori **Linux** (§4a)
- [ ] Lengkapi path kategori **Windows** (§4a), termasuk hard-deny list Windows
- [ ] Sembunyikan kategori khusus macOS (Xcode, simulator, Full Disk Access) di OS lain
- [ ] Ukuran di Windows pakai `len()` (`#[cfg(windows)]`)
- [ ] "Show in Finder" → "Show in Explorer"/"Show in Files" sesuai OS
- [ ] GitHub Actions dengan `tauri-action`: build macOS (Apple Silicon), Windows, Linux
- [ ] Jalankan unit test di ketiga OS di CI

**Verifikasi:**
- [ ] Build CI hijau untuk ketiga OS
- [ ] (Jika ada akses) smoke test manual di Windows/Linux dengan dry-run

---

## M8 — Polish & Rilis

- [ ] Icon app (`pnpm tauri icon`)
- [ ] Empty states: belum scan, tidak ada yang bisa dibersihkan, belum ada root folder, izin kurang
- [ ] Loading skeleton untuk kartu kategori
- [ ] Format waktu relatif ("3 bulan lalu") & ukuran basis 1000 (GB)
- [ ] Aksesibilitas dasar: navigasi keyboard, label checkbox, kontras warna
- [ ] Batasi `capabilities/default.json` seminimal mungkin; pastikan frontend tidak punya akses `fs` langsung
- [ ] README: cara dev & build, daftar kategori + risikonya, tips menjaga `src-tauri/target` tetap kecil (`cargo clean` / `cargo-sweep`)
- [ ] Ukur: waktu buka, RAM idle, ukuran bundle → catat di README

**Verifikasi — Acceptance Criteria (§14):**
- [ ] App terbuka < 2 detik, RAM idle < 100 MB, bundle macOS < 20 MB
- [ ] Kapasitas disk sama dengan Finder (selisih < 1%)
- [ ] Scan dengan progress dan bisa dibatalkan
- [ ] Ukuran `node_modules` pnpm tidak dilebih-lebihkan
- [ ] Frontend tidak bisa mengirim path arbitrer untuk dihapus
- [ ] Item `trash` bisa dikembalikan dari Trash
- [ ] Dry-run tidak mengubah file apa pun
- [ ] Kegagalan per item tidak menghentikan proses
- [ ] Banner Full Disk Access berfungsi (macOS)
- [ ] Build sukses di macOS, Windows, Linux

---

## Catatan & Keputusan

> Agent: tulis di sini setiap keputusan teknis yang tidak tercakup brief, beserta alasannya.

| Tanggal | Keputusan | Alasan |
|---|---|---|
| 2026-10-07 | Penggunaan CSS native modern dengan custom tokens & variabel CSS | Memastikan app tetap ringan, startup cepat (< 2s), kontrol penuh atas layout desktop native, dan tanpa overhead build Tailwind CSS. |
| 2026-10-07 | Sinkronisasi tipe manual di `types.ts` vs `tauri-specta` | Tipe frontend ditulis rapi di `src/lib/types.ts` dengan camelCase mapping serde yang jelas, menjaga build time cepat tanpa dependency macro code-gen eksternal. |

## Isu Terbuka / Blocker

- [ ] _(kosong)_
