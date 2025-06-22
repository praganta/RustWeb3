use std::env;
use std::fs;

use web3::contract::{Contract, Options};
use web3::types::{Address, U256};
use web3::transports::Http;
use web3::Web3;
use web3::signing::Key;
use web3::signing::{SecretKeyRef, SecretKey};


use dotenv::from_filename;
use serde_json::Value;


use hex;

use base64::{engine::general_purpose, Engine as _};

pub async fn send_to_blockchain(temp: f32, humidity: f32) -> Result<(), Box<dyn std::error::Error>> {
    from_filename("gnjr.env").ok();

    // Baca environment variables
    let rpc_url = env::var("RPC_URL")?;
    let private_key = env::var("PRIVATE_KEY")?;
    let contract_address = env::var("CONTRACT_ADDRESS")?;

    // Setup transport & web3 instance
    let transport = Http::new(&rpc_url)?;
    let web3 = Web3::new(transport);

    // Parse private key
    let key = private_key.strip_prefix("0x").unwrap_or(&private_key);
    let secret_key = SecretKey::from_slice(&hex::decode(key)?)?;
    let from = SecretKeyRef::new(&secret_key).address();
    println!("🔑 Using address: {:?}", from);

    // Load ABI JSON file and extract ABI array
    let abi_path = "./SensorStorage.json"; // atau path relatif sesuai tempat file ini
    let abi_str = fs::read_to_string(abi_path)?;
    let abi_json: Value = serde_json::from_str(&abi_str)?;
    let abi_bytes = serde_json::to_vec(&abi_json["abi"])?; // hanya bagian "abi" yang dibutuhkan

    // Parse contract address
    let contract_addr: Address = contract_address.parse()?;

    // Create contract instance
    let contract = Contract::from_json(web3.eth(), contract_addr, &abi_bytes)?;

    // Simpan data asli sebagai JSON string (bisa dibaca React)
    let json_string = format!(r#"{{"temperature": {:.1}, "humidity": {:.1}}}"#, temp, humidity);
    let data_hash = general_purpose::STANDARD.encode(json_string);  // encode agar bisa ditaruh di blockchain

    // Sensor ID bisa ditentukan dinamis
    let sensor_id = "SHT20-PascaPanen-001";

    // Kirim transaksi
    let tx_hash = contract
        .signed_call(
            "storeData",
            (sensor_id.to_string(), data_hash),
            Options {
                gas: Some(U256::from(300000)),
                ..Default::default()
          },  
          SecretKeyRef::new(&secret_key),
        )
        .await?;

    println!("📦 Transaction sent to blockchain! TX hash: {:?}", tx_hash);

    Ok(())
}
