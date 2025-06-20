import sys
import json
import asyncio
from PyQt5.QtWidgets import QApplication, QWidget, QVBoxLayout, QHBoxLayout, QPushButton, QLabel, QGridLayout
from PyQt5.QtCore import QThread, pyqtSignal
import pyqtgraph as pg
import websockets

# --- KONFIGURASI ---
WEBSOCKET_URL = "ws://localhost:8000/ws"
MAX_DATA_POINTS = 100 # Jumlah data sing ditampilno nang grafik

# --- THREAD UNTUK KONEKSI WEBSOCKET ---
class WebSocketThread(QThread):
    newData = pyqtSignal(float, float) # Sinyal kanggo (suhu, kelembapan)
    connectionStatus = pyqtSignal(str) # Sinyal kanggo status koneksi

    def __init__(self):
        super().__init__()
        self._is_running = True

    def run(self):
        # Njalanke loop event asyncio nang njero thread
        asyncio.run(self.listen())

    async def listen(self):
        self.connectionStatus.emit("Connecting...")
        print("Mencoba terhubung ke server WebSocket...")
        while self._is_running:
            try:
                async with websockets.connect(WEBSOCKET_URL) as websocket:
                    print("✅ Terhubung ke server WebSocket.")
                    self.connectionStatus.emit("Connected")
                    async for message in websocket:
                        if not self._is_running: break
                        try:
                            data = json.loads(message)
                            temp = data.get('temperature_celsius', 0.0)
                            humidity = data.get('humidity_percent', 0.0)
                            self.newData.emit(temp, humidity)
                        except json.JSONDecodeError:
                            print("⚠️ Pesan diterima bukan JSON.")
            except Exception as e:
                print(f"❌ Koneksi WebSocket gagal: {e}. Mencoba lagi dalam 5 detik...")
                self.connectionStatus.emit("Retrying...")
                await asyncio.sleep(5)
        
        print("🔌 Thread WebSocket dihentikan.")
        self.connectionStatus.emit("Stopped")
    
    def stop(self):
        self._is_running = False

# --- UI UTAMA APLIKASI ---
class ControllableDashboard(QWidget):
    def __init__(self):
        super().__init__()
        self.ws_thread = None # Thread durung digawe teko awal
        self.initUI()
        
    def initUI(self):
        self.setWindowTitle('Real-time Sensor Dashboard')
        self.setGeometry(100, 100, 800, 650)
        
        main_layout = QVBoxLayout()
        
        # --- KONTROL & STATUS ---
        control_layout = QHBoxLayout()
        self.start_button = QPushButton("🚀 Mulai Aliran Data")
        self.start_button.clicked.connect(self.start_connection)
        self.start_button.setStyleSheet("background-color: #28a745; color: white; font-weight: bold;")
        
        self.stop_button = QPushButton("🛑 Hentikan Aliran Data")
        self.stop_button.clicked.connect(self.stop_connection)
        self.stop_button.setEnabled(False) # Awalnya non-aktif
        self.stop_button.setStyleSheet("background-color: #dc3545; color: white; font-weight: bold;")

        self.status_label = QLabel("Status: Disconnected")
        self.status_label.setStyleSheet("font-weight: bold;")

        control_layout.addWidget(self.start_button)
        control_layout.addWidget(self.stop_button)
        control_layout.addStretch()
        control_layout.addWidget(self.status_label)
        main_layout.addLayout(control_layout)

        # --- Bagian Teks Data Saat Ini ---
        grid_layout = QGridLayout()
        self.temp_label = QLabel("Suhu: - °C")
        self.temp_label.setStyleSheet("font-size: 24px; color: #d9534f; font-weight: bold;")
        self.humidity_label = QLabel("Kelembapan: - %")
        self.humidity_label.setStyleSheet("font-size: 24px; color: #428bca; font-weight: bold;")
        grid_layout.addWidget(self.temp_label, 0, 0)
        grid_layout.addWidget(self.humidity_label, 0, 1)
        main_layout.addLayout(grid_layout)

        # --- Bagian Grafik ---
        self.graph_widget = pg.PlotWidget()
        self.graph_widget.setBackground('w')
        # ... (konfigurasi grafik liyane podo koyok sakdurunge)
        self.graph_widget.addLegend()
        self.time_data = list(range(MAX_DATA_POINTS))
        self.temp_data = [0] * MAX_DATA_POINTS
        self.humidity_data = [0] * MAX_DATA_POINTS
        self.temp_line = self.graph_widget.plot(self.time_data, self.temp_data, name="Suhu (°C)", pen=pg.mkPen(color='#d9534f', width=2))
        self.humidity_line = self.graph_widget.plot(self.time_data, self.humidity_data, name="Kelembapan (%)", pen=pg.mkPen(color='#428bca', width=2))
        main_layout.addWidget(self.graph_widget)

        self.setLayout(main_layout)

    def start_connection(self):
        if not self.ws_thread or not self.ws_thread.isRunning():
            self.ws_thread = WebSocketThread()
            self.ws_thread.newData.connect(self.update_data)
            self.ws_thread.connectionStatus.connect(self.update_status)
            self.ws_thread.start()
            
            self.start_button.setEnabled(False)
            self.stop_button.setEnabled(True)

    def stop_connection(self):
        if self.ws_thread and self.ws_thread.isRunning():
            self.ws_thread.stop()
            # Minta thread untuk keluar dari loop eventnya
            self.ws_thread.quit() 
            # Tunggu sampek bener-bener mandek
            self.ws_thread.wait() 
            
            self.start_button.setEnabled(True)
            self.stop_button.setEnabled(False)
            self.status_label.setText("Status: Stopped")
    
    def update_status(self, status_text):
        self.status_label.setText(f"Status: {status_text}")
    
    def update_data(self, temp, humidity):
        self.temp_label.setText(f"Suhu: {temp:.1f} °C")
        self.humidity_label.setText(f"Kelembapan: {humidity:.1f} %")
        self.temp_data = self.temp_data[1:] + [temp]
        self.humidity_data = self.humidity_data[1:] + [humidity]
        self.temp_line.setData(self.time_data, self.temp_data)
        self.humidity_line.setData(self.time_data, self.humidity_data)

    def closeEvent(self, event):
        # Mestekno thread mati pas jendela ditutup
        if self.ws_thread and self.ws_thread.isRunning():
            self.ws_thread.stop()
            self.ws_thread.quit()
            self.ws_thread.wait()
        event.accept()

if __name__ == '__main__':
    app = QApplication(sys.argv)
    dashboard = ControllableDashboard()
    dashboard.show()
    sys.exit(app.exec_())
