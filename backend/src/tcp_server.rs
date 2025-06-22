use chrono::DateTime;
use futures::stream;
use influxdb2::{models::DataPoint, Client};
use serde::Deserialize;
use std::error::Error;
use tokio::io::{AsyncBufReadExt, BufReader};
use tokio::net::TcpListener;
use std::sync::{Arc, Mutex};
use crate::SensorState;

#[derive(Debug, Deserialize)]
struct SensorData {
    timestamp: String,
    sensor_id: String,
    location: String,
    process_stage: String,
    temperature_celsius: f64,
    humidity_percent: f64,
}

pub async fn run_server(sensor_state: Arc<Mutex<SensorState>>) -> Result<(), Box<dyn Error>> {
    let token = "l-ymSI4CixCc_FFBv3t7aieq5WWF2ekb-R5KbP3RzDVdO89g1kOUtwiDYy4oNs6LYF_NpveFGltbe0CVg84kdQ==";
    let org = "gnjr";
    let bucket = "iyonjar";
    let client = Client::new("http://localhost:8086", org, token);

    let listener = TcpListener::bind("127.0.0.1:7878").await?;
    println!("📡 Server listening on port 7878");

    loop {
        let (socket, _) = listener.accept().await?;
        let client = client.clone();
        let bucket = bucket.to_string();
        let sensor_state = sensor_state.clone();

        tokio::spawn(async move {
            let reader = BufReader::new(socket);
            let mut lines = reader.lines();

            while let Ok(Some(line)) = lines.next_line().await {
                match serde_json::from_str::<SensorData>(&line) {
                    Ok(data) => {
                        println!("📥 Received: {:?}", data);

                        // ✅ Lock aman
                        match sensor_state.lock() {
                            Ok(mut state) => {
                                state.temperature = data.temperature_celsius as f32;
                                state.humidity = data.humidity_percent as f32;
                                println!("📡 Updated SensorState → Temp: {}, Hum: {}", state.temperature, state.humidity);
                            }
                            Err(e) => {
                                eprintln!("❌ Gagal lock SensorState: {}", e);
                                continue;
                            }
                        }

                        let parsed_time: DateTime<chrono::Utc> = match DateTime::parse_from_rfc3339(&data.timestamp) {
                            Ok(t) => t.with_timezone(&chrono::Utc),
                            Err(e) => {
                                eprintln!("⚠️  Invalid timestamp format: {}", e);
                                continue;
                            }
                        };

                        let point = match DataPoint::builder("environtment")
                            .tag("sensor_id", data.sensor_id)
                            .tag("location", data.location)
                            .tag("process_stage", data.process_stage)
                            .field("temperature", data.temperature_celsius)
                            .field("humidity", data.humidity_percent)
                            .timestamp(parsed_time.timestamp_nanos_opt().unwrap_or(0))
                            .build()
                        {
                            Ok(p) => p,
                            Err(e) => {
                                eprintln!("❌ Failed to build data point: {}", e);
                                continue;
                            }
                        };

                        let max_retries = 3;
                        for attempt in 1..=max_retries {
                            let point_clone = point.clone();
                            let result = client
                                .write(&bucket, stream::once(async move { point_clone }))
                                .await;

                            match result {
                                Ok(_) => {
                                    println!("✅ Data successfully written to InfluxDB");
                                    break;
                                }
                                Err(e) => {
                                    eprintln!(
                                        "❌ Attempt {}/{} - Failed to write to InfluxDB: {}",
                                        attempt, max_retries, e
                                    );
                                    if let Some(source) = e.source() {
                                        eprintln!("🔍 Error source: {}", source);
                                    }

                                    if attempt == max_retries {
                                        eprintln!(
                                            "🚨 Failed after {} retries. Skipping data point.",
                                            max_retries
                                        );
                                    } else {
                                        tokio::time::sleep(std::time::Duration::from_secs(2)).await;
                                    }
                                }
                            }
                        }
                    }
                    Err(e) => eprintln!("⚠️  Invalid JSON received: {}", e),
                }
            }
        });
    }
}
