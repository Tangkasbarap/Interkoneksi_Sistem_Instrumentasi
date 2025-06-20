use tokio_modbus::client::rtu::attach_slave;
use tokio_serial::{SerialPortBuilderExt, Parity, StopBits, DataBits};
use tokio_modbus::prelude::*;
use tokio::net::TcpStream;
use tokio::io::AsyncWriteExt;
use serde::Serialize;
use chrono::Utc;
use std::error::Error;
use tokio::time::{sleep, Duration};

#[derive(Serialize)]
struct SensorData {
    timestamp: String, 
    sensor_id: String, 
    location: String, 
    process_stage: String,
    temperature_celsius: f32, 
    humidity_percent: f32,
}

async fn read_sensor(slave: u8) -> Result<Vec<u16>, Box<dyn Error>> {
    let builder = tokio_serial::new("/dev/ttyUSB0", 9600)
        .parity(Parity::None)
        .stop_bits(StopBits::One)
        .data_bits(DataBits::Eight)
        .timeout(std::time::Duration::from_secs(1));
    let port = builder.open_native_async()?;
    let slave  = Slave(slave);
    let mut ctx = attach_slave(port, slave);
    let response = ctx.read_input_registers(1, 2).await?;
    Ok(response)
}

#[tokio::main]
async fn main() -> Result<(), Box<dyn Error>> {
    loop {
        match read_sensor(1).await {
            Ok(response) if response.len() == 2 => {
                let temp = response[0] as f32 / 10.0;
                let rh = response[1] as f32 / 10.0;
                println!("Temperature : {:.1} °C | HUmidity : {:.1} %", temp, rh);

                let data = SensorData {
                    timestamp: Utc::now().to_rfc3339(), sensor_id: "SHT20-PascaPanen-001".into(),
                    location: "Gudang Fermentasi 1".into(), process_stage: "Fermentasi".into(),
                    temperature_celsius: temp, humidity_percent: rh,
                };
                let json = serde_json::to_string(&data)?;
                
                match TcpStream::connect("127.0.0.1:9000").await {
                    Ok(mut stream) => {
                        stream.write_all(json.as_bytes()).await?;
                        stream.write_all(b"\n").await?;
                        println!("✅ Data dikirim ke TCP server");
                    },
                    Err(e) => println!("❌ Gagal konek ke TCP server: {}", e),
                }
            },
            Ok(other) => println!("⚠️ Data tidak lengkap: {:?}", other),
            Err(e) => println!("❌ Gagal baca sensor: {}", e),
        }
        sleep(Duration::from_secs(5)).await;
    }
}
