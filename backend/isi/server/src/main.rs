// --- BAGIAN 1: IMPORTS ---
use axum::{
    extract::{ws::{Message, WebSocket}, State, WebSocketUpgrade, Query},
    http::StatusCode,
    response::{IntoResponse, Json},
    routing::get,
    Router,
};
use ethers::prelude::*;
use futures::{sink::SinkExt, stream::StreamExt};
use influxdb2::{models::DataPoint, Client as InfluxClient};
use serde::{Deserialize, Serialize};
use std::{collections::HashMap, env, fs, net::SocketAddr, str::FromStr, sync::Arc};
use tokio::sync::Mutex;
use tokio::net::{TcpListener, TcpStream};
use tokio::io::{AsyncBufReadExt, BufReader};
use tower_http::cors::{Any, CorsLayer};
use uuid::Uuid;

// --- BAGIAN 2: DEFINISI STRUCT & TYPES ---
#[derive(Deserialize, Serialize, Debug, Clone)]
struct SensorData {
    timestamp: String,
    sensor_id: String,
    location: String,
    process_stage: String,
    temperature_celsius: f32,
    humidity_percent: f32,
}

#[derive(Deserialize)]
struct VerifyParams {
    tx_hash: String,
}

#[derive(Deserialize)]
struct ContractInfo {
    address: String,
}

#[derive(Serialize)]
struct ApiResponse {
    message: String,
}

type WsClients = Arc<Mutex<HashMap<String, tokio::sync::mpsc::Sender<Message>>>>;

#[derive(Clone)]
struct AppState {
    ethers_provider: Arc<Provider<Http>>,
    contract_address: Arc<H160>,
    ws_clients: WsClients,
    influx_client: Arc<InfluxClient>,
    influx_bucket: Arc<String>,
}

// --- BAGIAN 3: FUNGSI-FUNGSI LOGIKA ---

async fn write_to_influxdb(client: &InfluxClient, bucket: &str, data: &SensorData) -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
    let timestamp = chrono::DateTime::parse_from_rfc3339(&data.timestamp)?.timestamp_nanos_opt().ok_or("Timestamp conversion failed")?;
    let point = DataPoint::builder("monitoring")
        .tag("sensor_id", &data.sensor_id)
        .tag("location", &data.location)
        .tag("stage", &data.process_stage)
        .field("temperature", data.temperature_celsius as f64)
        .field("humidity", data.humidity_percent as f64)
        .timestamp(timestamp)
        .build()?;
    client.write(bucket, futures::stream::iter(vec![point])).await?;
    Ok(())
}

async fn process_socket(stream: TcpStream, app_state: AppState) {
    let mut reader = BufReader::new(stream);
    let mut line_buffer = String::new();
    loop {
        if reader.read_line(&mut line_buffer).await.unwrap_or(0) == 0 { break; }
        let trimmed_line = line_buffer.trim();
        if let Ok(data) = serde_json::from_str::<SensorData>(trimmed_line) {
            println!("[TCP] 📥 Data diterima: {}", data.sensor_id);
            if let Err(e) = write_to_influxdb(&app_state.influx_client, &app_state.influx_bucket, &data).await {
                eprintln!("[INFLUX] ❌ Gagal tulis ke DB: {}", e);
            }
            let data_json = serde_json::to_string(&data).unwrap();
            let clients = app_state.ws_clients.lock().await;
            for sender in clients.values() {
                if sender.send(Message::Text(data_json.clone())).await.is_err() {
                    // Klien mungkin terputus, akan dihapus saat koneksinya benar-benar tertutup
                }
            }
        } else if !trimmed_line.is_empty() {
            eprintln!("[TCP] ⚠️ Gagal parse JSON dari klien: {}", trimmed_line);
        }
        line_buffer.clear();
    }
}

async fn tcp_listener_task(app_state: AppState) {
    let tcp_addr = env::var("TCP_SERVER_ADDRESS").unwrap();
    let listener = TcpListener::bind(&tcp_addr).await.unwrap();
    println!("[TCP] 🚀 Server TCP berjalan di {}", tcp_addr);
    loop {
        let (socket, addr) = listener.accept().await.unwrap();
        println!("[TCP] 🤝 Koneksi baru dari: {}", addr);
        tokio::spawn(process_socket(socket, app_state.clone()));
    }
}

async fn verify_access_handler(State(state): State<AppState>, Query(params): Query<VerifyParams>) -> impl IntoResponse {
    println!("[API] 📞 Permintaan verifikasi diterima: {}", params.tx_hash);
    let tx_hash = match H256::from_str(&params.tx_hash) {
        Ok(h) => h,
        Err(_) => return (StatusCode::BAD_REQUEST, Json(ApiResponse { message: "Format tx_hash tidak valid.".to_string() })).into_response(),
    };
    if let Some(r) = state.ethers_provider.get_transaction_receipt(tx_hash).await.unwrap_or(None) {
        if r.status.unwrap_or_default().as_u64() == 1 && r.to.unwrap_or_default() == *state.contract_address {
            println!("[API] ✅ Verifikasi transaksi BERHASIL: {}", params.tx_hash);
            return (StatusCode::OK, Json(ApiResponse { message: "Verifikasi berhasil. Silakan hubungkan WebSocket.".to_string() })).into_response();
        }
    }
    println!("[API] ❌ Verifikasi transaksi GAGAL: {}", params.tx_hash);
    (StatusCode::UNAUTHORIZED, Json(ApiResponse { message: "Bukti pembayaran tidak valid atau tidak ditemukan.".to_string() })).into_response()
}

async fn websocket_handler(ws: WebSocketUpgrade, State(state): State<AppState>) -> impl IntoResponse {
    println!("[API] Menerima permintaan upgrade ke WebSocket...");
    ws.on_upgrade(move |socket| handle_websocket(socket, state))
}
// Ganti seluruh fungsi handle_websocket dengan ini

async fn handle_websocket(socket: WebSocket, state: AppState) {
    let (mut sender, mut receiver) = socket.split();
    let client_id = Uuid::new_v4().to_string();
    let (tx, mut rx) = tokio::sync::mpsc::channel(100);

    state.ws_clients.lock().await.insert(client_id.clone(), tx);
    println!("[WS] 🚀 Klien WebSocket baru terhubung: {}", client_id);

    // --- BAGIAN YANG DIPERBAIKI ---
    // 1. Buat klon/duplikat dari client_id sebelum dipindahkan ke dalam task.
    let client_id_clone_for_task = client_id.clone();

    tokio::spawn(async move {
        while let Some(msg_to_send) = rx.recv().await {
            if sender.send(msg_to_send).await.is_err() {
                // 2. Gunakan klon di dalam task ini.
                println!("[WS] Gagal mengirim pesan ke klien {}, koneksi mungkin sudah terputus.", client_id_clone_for_task);
                break;
            }
        }
    });
    // --- AKHIR BAGIAN YANG DIPERBAIKI ---

    // Loop ini akan berjalan selama klien masih terhubung
    while let Some(Ok(_)) = receiver.next().await {}

    // Setelah loop di atas selesai (klien terputus), kita hapus dari daftar.
    // Kita menggunakan `client_id` yang asli di sini, yang kepemilikannya tidak pernah pindah.
    state.ws_clients.lock().await.remove(&client_id);
    println!("[WS] 🔌 Klien WebSocket terputus: {}", client_id);
}
async fn api_server_task(app_state: AppState) {
    let api_addr: SocketAddr = env::var("API_SERVER_ADDRESS").unwrap().parse().unwrap();
    let cors = CorsLayer::new().allow_origin(Any).allow_methods(Any).allow_headers(Any);
    let app = Router::new()
        .route("/verify-access", get(verify_access_handler))
        .route("/ws", get(websocket_handler))
        .with_state(app_state).layer(cors);
    println!("[API] 🚀 Server API & WebSocket berjalan di http://{}", api_addr);
    let listener = tokio::net::TcpListener::bind(api_addr).await.unwrap();
    axum::serve(listener, app).await.unwrap();
}

// --- BAGIAN 4: FUNGSI MAIN ---
#[tokio::main]
async fn main() {
    dotenvy::dotenv().expect("File .env tidak ditemukan");
    
    let contract_info_str = fs::read_to_string("../../../deployedAddress.json")
        .expect("File deployedAddress.json tidak ditemukan. Pastikan path relatif sudah benar dan Anda sudah menjalankan deployment Hardhat.");
    let contract_info: ContractInfo = serde_json::from_str(&contract_info_str).expect("Gagal parse deployedAddress.json");

    let app_state = AppState {
        ethers_provider: Arc::new(Provider::<Http>::try_from(env::var("GANACHE_URL").unwrap()).unwrap()),
        contract_address: Arc::new(H160::from_str(&contract_info.address).unwrap()),
        influx_client: Arc::new(InfluxClient::new(env::var("INFLUXDB_URL").unwrap(), env::var("INFLUXDB_ORG").unwrap(), env::var("INFLUXDB_TOKEN").unwrap())),
        influx_bucket: Arc::new(env::var("INFLUXDB_BUCKET").unwrap()),
        ws_clients: Arc::new(Mutex::new(HashMap::new())),
    };

    println!("🔥 Memulai semua service...");
    tokio::join!(
        tcp_listener_task(app_state.clone()),
        api_server_task(app_state.clone())
    );
}
