## Project Kakao TRace 
Proyek ini adalah sebuah sistem end-to-end yang mendemonstrasikan bagaimana data dari sensor fisik (IoT) dapat dikumpulkan, disimpan, dan aksesnya diperjualbelikan secara terdesentralisasi menggunakan teknologi blockchain.
Sistem ini menggabungkan backend berperforma tinggi yang ditulis dalam Rust, database time-series InfluxDB, smart contract di atas blockchain lokal (Ganache), dan dua jenis antarmuka klien: aplikasi web (React) dan dashboard desktop (PyQt5).
## Arsitektur Sistem
* Arsitektur sistem ini bersifat hybrid, memanfaatkan keunggulan dari teknologi on-chain (blockchain) dan off-chain (server & database tradisional) untuk mencapai efisiensi dan keamanan.
  * Lapisan Pengumpulan Data (Off-Chain): Sensor fisik dibaca oleh klien Rust (sensor) dan datanya dikirim melalui TCP ke server utama.

  * Lapisan Backend & Penyimpanan (Off-Chain): Server utama (server) yang ditulis dalam Rust menerima data, menyimpannya di InfluxDB untuk analisis historis, dan menyiarkannya secara real-time melalui WebSocket.

  * Lapisan Transaksi & Kepercayaan (On-Chain): Smart Contract yang di-deploy di blockchain Ganache bertindak sebagai "kasir digital". Ia mengelola hak akses dan pembayaran secara transparan.

  * Lapisan Presentasi (Klien): Pengguna berinteraksi dengan sistem melalui Aplikasi Web (React) atau Dashboard Desktop (PyQt5). Mereka melakukan pembayaran melalui MetaMask untuk mendapatkan hak akses melihat data real-time.
  
![WhatsApp Image 2025-06-23 at 00 50 34](https://github.com/user-attachments/assets/d2c78f6b-a7d0-4d29-9eaa-a665225f681b)



