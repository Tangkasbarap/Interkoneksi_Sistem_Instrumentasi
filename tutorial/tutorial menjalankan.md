# Tutorial Lengkap Menjalankan Project Kakao TRace
Dokumen ini adalah panduan langkah-demi-langkah untuk menginstal, mengkonfigurasi, dan menjalankan keseluruhan sistem Marketplace Data Sensor IoT. Tutorial ini akan memandu Anda melalui setiap komponen, dari backend hingga antarmuka pengguna.

## Prasyarat
Sebelum memulai, pastikan semua perangkat lunak esensial sudah terinstal di sistem operasi Anda (direkomendasikan Ubuntu/Linux):

- Node.js dan npm

- Rust dan Cargo

- Python 3 dan pip

- Ganache (sudo npm install -g ganache)

- InfluxDB (sudah terinstal dan service berjalan)

- bEkstensi Browser MetaMask

### Memulai Fondasi (Blockchain dengan Hardhat)
Buka Terminal 1 - Jalankan GANACHE:
Jalankan perintah ini untuk memulai blockchain lokal. Opsi -m dengan mnemonic yang sama memastikan Anda selalu mendapatkan 10 akun yang sama setiap saat, lengkap dengan saldo 100 ETH untuk development. Biarkan terminal ini tetap terbuka.

```   $ ganache -m "myth like bonus scare over problem client lizard pioneer submit female collect"   ```

Buka Terminal 2 - Lakukan DEPLOYMENT dengan HARDHAT:
Setiap kali setelah Ganache dimulai, kita harus men-deploy ulang kontrak kita untuk mendapatkan alamat yang baru dan valid. Perintah npx hardhat run adalah cara kita menyuruh Hardhat untuk menjalankan skrip.

## Masuk ke folder blockchain proyek Anda

``` $ cd Kelompok9/blockchain ```

Jalankan skrip deployment menggunakan Hardhat ke jaringan localhost

``` $ npx hardhat run scripts/deploy.js --network localhost  ```

Tunggu hingga muncul pesan ✅ Alamat & ABI disimpan di deployedAddress.json. Ini artinya "peta" alamat kontrak Anda sudah diperbarui dan siap digunakan oleh komponen lain.

### Menghidupkan Otak Proyek (Backend)
Buka Terminal 3 - Jalankan SERVER RUST:
Server ini perlu dijalankan setelah proses deployment selesai agar ia dapat membaca file deployedAddress.json yang terbaru untuk mengetahui alamat kontrak yang benar.

## Masuk ke folder server backend

``` $ cd Kelompok9/backend/isi/server ```

Jalankan program server dengan Cargo

``` cargo run ```

Tunggu hingga Anda melihat pesan konfirmasi di terminal, seperti [TCP] Server TCP berjalan... dan [API] Server API & WebSocket berjalan.... Biarkan terminal ini tetap terbuka.

Buka Terminal 4 - Jalankan KLIEN SENSOR:
Setelah server utama siap, kita bisa menghidupkan "petani" yang akan mengirimkan data.

## Masuk ke folder klien sensor

``` $ cd Kelompok9/backend/isi/sensor ```

Jalankan program klien sensor dengan Cargo

``` cargo run ```

Anda akan mulai melihat log pembacaan suhu dan kelembapan di terminal ini. Secara bersamaan, di Terminal 3 (Server), Anda juga akan melihat log bahwa data diterima dan disimpan.

### Membuka Toko (Klien/Antarmuka Pengguna)
Tujuan: Menjalankan antarmuka pengguna di mana interaksi akhir (pembelian dan visualisasi data) terjadi.

Buka Terminal 5 - Jalankan ANTARMUKA PENGGUNA:
Anda memiliki dua pilihan antarmuka. Jalankan salah satu (atau keduanya di terminal yang berbeda).

Pilihan 1: Aplikasi Web React

## Masuk ke folder frontend web

``` cd Kelompok9/frontend ```

Jalankan server pengembangan React

``` npm start ```

Sebuah tab browser akan otomatis terbuka ke alamat http://localhost:3000 (atau port lain yang tersedia).

Pilihan 2: Dashboard Desktop PyQt5

## Masuk ke folder dashboard desktop

``` $ cd Kelompok9/desktop_dashboard ```

Aktifkan lingkungan virtual Python

``` source venv/bin/activate ```

Jalankan aplikasi desktop

``` python3 dashboard.py ```

Sebuah jendela aplikasi desktop akan muncul di layar Anda.

#### Alur Pengujian Akhir dari Sisi Pengguna
Setelah semua komponen berjalan, ini adalah alur yang akan dialami oleh pengguna:

- Hubungkan Wallet: Di aplikasi klien (web atau desktop), pengguna mengklik tombol "Hubungkan Wallet".

- Setujui di MetaMask: Sebuah pop-up MetaMask akan muncul. Pastikan MetaMask Anda berada di jaringan Ganache dan akun "Imported" (yang memiliki saldo 100 ETH) yang terpilih. Setujui koneksi.

- Lakukan Pembelian: Setelah terhubung, pengguna mengklik tombol "Beli Akses & Tampilkan Data".

- Konfirmasi Transaksi: Setujui transaksi pembayaran yang muncul di pop-up MetaMask.

- Proses Otomatis: Setelah transaksi berhasil, aplikasi akan secara otomatis melakukan verifikasi ke backend dan kemudian membuka koneksi WebSocket.

- Selesai! Pengguna sekarang akan melihat data sensor mengalir secara real-time ke dashboard mereka, baik dalam bentuk kartu data maupun grafik yang terus diperbarui.
