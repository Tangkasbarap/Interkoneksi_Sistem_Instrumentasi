import React, { useState, useEffect } from 'react';
import { ethers } from 'ethers';
import './App.css';

// Path ke file konfigurasi di dalam folder public
const contractConfigFile = '/deployedAddress.json';

function App() {
  // --- State Management ---
  const [contractAddress, setContractAddress] = useState('');
  const [contractABI, setContractABI] = useState([]);
  const [account, setAccount] = useState(null);
  const [contract, setContract] = useState(null);
  const [isVerified, setIsVerified] = useState(false);
  const [realtimeData, setRealtimeData] = useState([]);
  const [error, setError] = useState('');
  const [loadingMessage, setLoadingMessage] = useState('Memuat Konfigurasi Kontrak...');
  const [wsStatus, setWsStatus] = useState('Disconnected');

  // --- Hooks ---
  // 1. useEffect untuk memuat info kontrak dari /public saat aplikasi pertama kali dimuat
  useEffect(() => {
    const loadContractInfo = async () => {
      try {
        const response = await fetch(contractConfigFile);
        if (!response.ok) {
          throw new Error("Gagal memuat deployedAddress.json. Pastikan file ada di folder /public dan skrip deployment Hardhat sudah dijalankan.");
        }
        const data = await response.json();
        setContractAddress(data.address);
        setContractABI(data.abi);
        console.log("Info kontrak berhasil dimuat:", data.address);
      } catch (err) {
        console.error(err);
        setError(err.message);
      } finally {
        setLoadingMessage(''); // Selesai memuat konfigurasi
      }
    };
    loadContractInfo();
  }, []);

  // 2. useEffect untuk mengelola koneksi WebSocket
  useEffect(() => {
    if (!isVerified) return;

    console.log("Membuka koneksi WebSocket...");
    setWsStatus('Connecting...');
    const ws = new WebSocket("ws://localhost:8000/ws");

    ws.onopen = () => { console.log("✅ Koneksi WebSocket terbuka!"); setWsStatus('Connected'); setError(''); };
    ws.onmessage = (event) => {
      try {
        const sensorData = JSON.parse(event.data);
        console.log("📥 Data real-time diterima:", sensorData);
        setRealtimeData(prevData => [sensorData, ...prevData.slice(0, 19)]);
      } catch (e) { console.error("Gagal parse data WebSocket:", e); }
    };
    ws.onerror = (err) => { console.error("Error WebSocket:", err); setError("Koneksi data real-time bermasalah."); setWsStatus('Error'); };
    ws.onclose = () => { console.log("🔌 Koneksi WebSocket ditutup."); setWsStatus('Disconnected'); setIsVerified(false); };
    
    return () => { ws.close(); };
  }, [isVerified]);

  // --- Fungsi Logika ---
  const connectWallet = async () => {
    if (!contractAddress || contractABI.length === 0) { setError("Info kontrak belum siap."); return; }
    if (typeof window.ethereum === 'undefined') { setError("MetaMask tidak terdeteksi."); return; }
    try {
      const provider = new ethers.BrowserProvider(window.ethereum);
      const accounts = await provider.send("eth_requestAccounts", []);
      const signer = await provider.getSigner();
      setAccount(accounts[0]);
      const marketplaceContract = new ethers.Contract(contractAddress, contractABI, signer);
      setContract(marketplaceContract);
    } catch (err) { setError("Gagal menghubungkan wallet: " + err.message); }
  };

  const handlePurchaseAndVerify = async () => {
    if (!contract) { setError("Wallet belum terhubung."); return; }
    setLoadingMessage('Mempersiapkan transaksi...'); setError(''); setRealtimeData([]); setIsVerified(false);
    try {
      const price = await contract.accessPrice();
      const tx = await contract.purchaseAccess({ value: price });
      setLoadingMessage('Menunggu konfirmasi blockchain...');
      await tx.wait();
      setLoadingMessage('Memverifikasi pembelian...');
      const response = await fetch(`http://localhost:8000/verify-access?tx_hash=${tx.hash}`);
      const result = await response.json();
      if (!response.ok) { throw new Error(result.message); }
      setIsVerified(true);
    } catch (err) {
      setError("Proses Gagal: " + (err.reason || err.message));
    } finally {
      setLoadingMessage('');
    }
  };

return (
  <div className="container">
    <h1>Kakao TRace</h1>

    {error && (
      <div className="error-box">
        <strong>Error:</strong> {error}
      </div>
    )}

    {!account ? (
      <button onClick={connectWallet} disabled={!!loadingMessage || !contractAddress}>
        {loadingMessage || 'Hubungkan Wallet MetaMask'}
      </button>
    ) : (
      <p>✅ <strong>Wallet Terhubung:</strong> {account}</p>
    )}

    {account && (
      <div>
        <button onClick={handlePurchaseAndVerify} disabled={!!loadingMessage}>
          {loadingMessage || 'Beli Akses & Tampilkan Data'}
        </button>
      </div>
    )}

    {isVerified && (
      <div className="data-box">
        <h2>📈 Aliran Data Sensor Real-time (Status: {wsStatus})</h2>
        {realtimeData.length === 0 && wsStatus === 'Connected' ? (
          <p>Menunggu data pertama dari server...</p>
        ) : (
          realtimeData.map((data, index) => (
            <pre key={data.timestamp + index}>
              {JSON.stringify(data, null, 2)}
            </pre>
          ))
        )}
      </div>
    )}
  </div>
);
}

export default App;
