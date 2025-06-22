use tokio_modbus::prelude::*;
use tokio_serial::SerialStream;
use std::time::Duration;
use tokio::net::TcpStream;
use tokio::io::AsyncWriteExt;
use chrono::Utc;
use serde::Serialize;

#[derive(Serialize)]
struct SensorPayload {
    timestamp: String,
    sensor_id: String,
    location: String,
    process_stage: String,
    temperature_celsius: f32,
    humidity_percent: f32,
}


pub async fn run_client() -> Result<(), Box<dyn std::error::Error>> {
    let builder = tokio_serial::new("/dev/ttyUSB0", 9600)
        .timeout(Duration::from_secs(1))
        .parity(tokio_serial::Parity::None)
        .stop_bits(tokio_serial::StopBits::One)
        .data_bits(tokio_serial::DataBits::Eight)
        .flow_control(tokio_serial::FlowControl::None);

    let port = SerialStream::open(&builder)?;
    let mut ctx = rtu::connect_slave(port, Slave(0x01)).await?;

    // Koneksi ke TCP server
    let mut stream = TcpStream::connect("127.0.0.1:7878").await?;
    println!("Connected to TCP server.");

    loop {
        let response = ctx.read_input_registers(0x0001, 2).await?;
        let raw_temp = response[0];
        let raw_humi = response[1];

        let temperature = raw_temp as f32 / 10.0;
        let humidity = raw_humi as f32 / 10.0;

        println!("Temperature: {} °C | Humidity: {} %", temperature, humidity);

        let payload = SensorPayload {
            timestamp: Utc::now().to_rfc3339(),
            sensor_id: "SHT20-PascaPanen-001".to_string(),
            location: "Gudang Fermentasi 1".to_string(),
            process_stage: "Fermentasi".to_string(),
            temperature_celsius: temperature,
            humidity_percent: humidity,
        };

        let json_payload = serde_json::to_string(&payload)?;
        stream.write_all(json_payload.as_bytes()).await?;
        stream.write_all(b"\n").await?; // Penting! agar TCP server bisa baca per baris

        tokio::time::sleep(Duration::from_secs(5)).await;
        
    }
}
