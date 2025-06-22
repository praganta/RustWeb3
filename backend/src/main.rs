mod sht20;
mod tcp_server;
mod blockchain;

use std::error::Error;
use std::sync::{Arc, Mutex};
use tokio::time::{sleep, Duration};

#[derive(Clone, Debug)]
pub struct SensorState {
    pub temperature: f32,
    pub humidity: f32,
}

#[tokio::main]
async fn main() -> Result<(), Box<dyn Error>> {
    let sensor_state = Arc::new(Mutex::new(SensorState {
        temperature: 0.0,
        humidity: 0.0,
    }));

    let state_for_tcp = sensor_state.clone();
    let state_for_blockchain = sensor_state.clone();

    // Jalankan TCP server
    tokio::spawn(async move {
        if let Err(e) = tcp_server::run_server(state_for_tcp).await {
            eprintln!("TCP Server error: {}", e);
        }
    });

    // Jalankan client pembaca sensor
    tokio::spawn(async {
        if let Err(e) = sht20::run_client().await {
            eprintln!("Sensor client error: {}", e);
        }
    });

    // Delay awal sensorstate
    sleep(Duration::from_secs(3)).await;

    // Kirim ke blockchain setiap 10 detik
    loop {
        match state_for_blockchain.lock() {
            Ok(state) => {
                let temperature = state.temperature;
                let humidity = state.humidity;
                if temperature == 0.0 && humidity == 0.0 {
                    println!("⚠️  Skip sending 0, 0 to blockchain");
                } else {
                    println!("🔁 Kirim ke blockchain: {}, {}", temperature, humidity);

                    match blockchain::send_to_blockchain(temperature, humidity).await {
                        Ok(_) => println!("📦 Data sent to blockchain"),
                        Err(e) => eprintln!("❌ Error sending to blockchain: {}", e),
                    }
                }
            }
            Err(e) => {
                eprintln!("❌ Failed to lock sensor state for blockchain: {}", e);
            }
        }

        sleep(Duration::from_secs(10)).await;
    }
}
