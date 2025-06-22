# Rust Project Monitoring Suhu & Kelembapan Tape (Web3 + Blockchain + Rust + Vite React)

## Teknologi :
- Sensor SHT20
- Rust (Modbus, Tcp Server, Web3)
- InfluxDB
- React (Vite)
- Smart Contract Solidity (Ganache)

# Cara menjalankan 
1. buka terminal ganache
2. buka terminal baru cd mytruffle, truffle migrate --reset
3. Copy hasl ABI ke frontend & Backend
   cp build/contracts/SensorStorage.json ../dapp_frontend/src
   cp build/contracts/SensorStorage.json ../backend/

4. update gnjr.env
   RPC_URL = http:127.0.0.1:8545
   Private_key = 0x... (Ambil akun 0 di terminal ganache)
   Contract_Address = 0x.. (ambil hasil deploy dari terminal truffle)

5. biarkan ganache tetap jalan dan buka terminal baru yaitu influxd (untuk tcp_server agar konek ke influxdb)
6. Jalankan Backend Rust
   cd backend
   cargo run
7. Jalankan Frontend React (Vite)
   cd dapp_frontend
   npm run dev
8. kalau mau lihat GUI QT tinggal buka terminal
   cd qt_gui
   ubah dulu environtmentnya
   source gnjrenv/bin/activate
   python main.py
9. kalau mau ngehubungin ke grafana tinggal buka terminal lagi
   sudo systemctl start grafana-server
