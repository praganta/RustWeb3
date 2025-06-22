import os
import sys
import PySide6

from PySide6.QtWidgets import (
    QApplication, QLabel, QVBoxLayout, QWidget, QPushButton, QHBoxLayout
)
from PySide6.QtCore import QTimer
from influxdb_client import InfluxDBClient
from matplotlib.backends.backend_qtagg import FigureCanvasQTAgg as FigureCanvas
from matplotlib.figure import Figure

# Setup plugin path (khusus Linux)
os.environ["QT_QPA_PLATFORM_PLUGIN_PATH"] = os.path.join(
    os.path.dirname(PySide6.__file__), "plugins", "platforms"
)

# Konfigurasi koneksi InfluxDB
INFLUX_URL = "http://localhost:8086"
TOKEN = "l-ymSI4CixCc_FFBv3t7aieq5WWF2ekb-R5KbP3RzDVdO89g1kOUtwiDYy4oNs6LYF_NpveFGltbe0CVg84kdQ=="
ORG = "gnjr"
BUCKET = "iyonjar"

# Batas alarm (opsional)
TEMP_THRESHOLD = 35.0  # derajat Celsius
HUM_THRESHOLD = 80.0   # persen

client = InfluxDBClient(url=INFLUX_URL, token=TOKEN, org=ORG)
query_api = client.query_api()


class MainWindow(QWidget):

    def resizeEvent(self, event):
        super().resizeEvent(event)
        self.canvas.draw()

    def __init__(self):
        super().__init__()
        self.setWindowTitle("Smart Fermentation Monitor")
        self.resize(600, 400)

        # Label utama
        self.temp_label = QLabel("Suhu: -- °C")
        self.hum_label = QLabel("Kelembaban: -- %")
        self.status_label = QLabel("Status: ❌ Disconnected")
        self.alarm_label = QLabel("")  # Kosong, isi hanya saat alarm

        #Grafik
        self.figure = Figure(figsize=(5,3), constrained_layout=True)
        self.canvas = FigureCanvas(self.figure)
        self.ax = self.figure.add_subplot(111)
        self.ax.set_title("Grafik Suhu & Kelembaban")
        self.ax.set_xlabel("Waktu")
        self.ax.set_ylabel("Nilai")
        self.ax.tick_params(axis='x', rotation=45)




        # Tombol kontrol
        self.start_button = QPushButton("▶️ Start")
        self.stop_button = QPushButton("⏹ Stop")
        self.stop_button.setEnabled(False)

        self.start_button.clicked.connect(self.start_monitoring)
        self.stop_button.clicked.connect(self.stop_monitoring)

        # Layout tombol horizontal
        btn_layout = QHBoxLayout()
        btn_layout.addWidget(self.start_button)
        btn_layout.addWidget(self.stop_button)

        # Layout utama
        layout = QVBoxLayout()
        layout.addWidget(self.temp_label)
        layout.addWidget(self.hum_label)
        layout.addWidget(self.status_label)
        layout.addWidget(self.alarm_label)
        layout.addWidget(self.canvas, stretch=1)
        layout.addLayout(btn_layout)
        self.setLayout(layout)

        # Timer auto-refresh
        self.timer = QTimer()
        self.timer.timeout.connect(self.update_data)

        # Tes koneksi awal
        self.test_connection()

    def test_connection(self):
        try:
            health = client.ping()
            if health:
                self.status_label.setText("Status: ✅ Connected to InfluxDB")
            else:
                self.status_label.setText("Status: ❌ Tidak bisa konek InfluxDB")
        except Exception as e:
            self.status_label.setText(f"Status: ❌ {str(e)}")

    def start_monitoring(self):
        self.timer.start(5000)
        self.start_button.setEnabled(False)
        self.stop_button.setEnabled(True)
        self.status_label.setText("Status: 🔄 Monitoring...")
        self.update_data()  # langsung panggil pertama

    def stop_monitoring(self):
        self.timer.stop()
        self.start_button.setEnabled(True)
        self.stop_button.setEnabled(False)
        self.status_label.setText("Status: ⏸️ Monitoring dihentikan")

    def update_data(self):
        query = f'''
        from(bucket: "{BUCKET}")
          |> range(start: -1m)
          |> filter(fn: (r) => r._measurement == "environtment")
          |> filter(fn: (r) => (r._field == "temperature" or r._field == "humidity"))
          |> sort(columns: ["_time"], desc: false)
          |> limit(n:10)
        '''
        try:
            result = query_api.query(query)
            temp_value = None
            hum_value = None

            for table in result:
                for record in table.records:
                    if record.get_field() == "temperature" and temp_value is None:
                        temp_value = record.get_value()
                    elif record.get_field() == "humidity" and hum_value is None:
                        hum_value = record.get_value()

            alarm_triggered = False

            if temp_value is not None:
                self.temp_label.setText(f"Suhu: {temp_value:.1f} °C")
                if temp_value > TEMP_THRESHOLD:
                    alarm_triggered = True

            if hum_value is not None:
                self.hum_label.setText(f"Kelembaban: {hum_value:.1f} %")
                if hum_value > HUM_THRESHOLD:
                    alarm_triggered = True

            if temp_value is None and hum_value is None:
                self.temp_label.setText("Suhu: -- °C")
                self.hum_label.setText("Kelembaban: -- %")

            # Ambil data untuk grafik
            temp_times = []
            temp_values = []
            hum_times = []
            hum_values = []

            for table in result:
                 for record in table.records:
                      ts = record.get_time().strftime("%H:%M:%S")
                      if record.get_field() == "temperature":
                         if temp_value is None:
                             temp_value = record.get_value()
                         temp_values.append(record.get_value())
                         temp_times.append(ts)
                      elif record.get_field() == "humidity":
                         if hum_value is None:
                             hum_value = record.get.value()
                         hum_values.append(record.get_value())
                         hum_times.append(ts)

            # Update grafik
            self.ax.clear()
            self.ax.plot(temp_times,temp_values, label="Suhu (°C)", color='red', marker='o')
            self.ax.plot(hum_times,hum_values , label="Kelembaban (%)", color='blue', marker='o')
            self.ax.set_title("Grafik Suhu & Kelembaban")
            self.ax.set_xlabel("Waktu")
            self.ax.set_ylabel("Nilai")
            self.ax.legend()
            self.ax.tick_params(axis='x', rotation=45)
            self.canvas.draw()

            if alarm_triggered:
                self.alarm_label.setText("⚠️ ALARM: Batas suhu/kelembaban terlampaui!")
                self.alarm_label.setStyleSheet("color: red; font-weight: bold;")
            else:
                self.alarm_label.setText("")
        except Exception as e:
            self.temp_label.setText("Suhu: -- (Error)")
            self.hum_label.setText("Kelembaban: -- (Error)")
            self.status_label.setText(f"Status: ❌ {str(e)}")


if __name__ == "__main__":
    app = QApplication(sys.argv)
    window = MainWindow()
    window.show()
    sys.exit(app.exec())
