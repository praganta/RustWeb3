import { useEffect, useState } from 'react';
import { ethers } from 'ethers';
import SensorStorage from './SensorStorage.json';
import {
  LineChart, Line, XAxis, YAxis, CartesianGrid, Tooltip, Legend, ResponsiveContainer
} from 'recharts';
import QRCode from 'react-qr-code';

const abi = SensorStorage.abi;
const CONTRACT_ADDRESS = '0xb441dF60b709bfc36E003Ea9be1E7a911DE72100'; // Contract selalu berubah harus dari truffle migrate 

function App() {
  const [temperature, setTemperature] = useState('--');
  const [humidity, setHumidity] = useState('--');
  const [account, setAccount] = useState('');
  const [error, setError] = useState('');
  const [view, setView] = useState('live');
  const [history, setHistory] = useState([]);
  const [datetime, setDatetime] = useState(new Date());

  const fetchData = async () => {
    try {
      if (!window.ethereum) return alert('Please install MetaMask!');
      const provider = new ethers.BrowserProvider(window.ethereum);
      const signer = await provider.getSigner();
      const contract = new ethers.Contract(CONTRACT_ADDRESS, abi, signer);
      const accounts = await window.ethereum.request({ method: 'eth_requestAccounts' });
      setAccount(accounts[0] || '');
      const countNumber = Number(await contract.getRecordCount());
      if (!countNumber) return setError("Belum ada data disimpan di blockchain.");

      const fetchedHistory = [];
      for (let i = Math.max(0, countNumber - 10); i < countNumber; i++) {
        const rec = await contract.getRecord(i);
        const parsed = JSON.parse(atob(rec[2]));
        fetchedHistory.push({
          index: i + 1,
          ...parsed,
          sensorId: rec[1],
          timestamp: new Date(Number(rec[0]) * 1000).toLocaleString()
        });
      }

      const logs = await contract.queryFilter('DataStored', 0, 'latest');
      fetchedHistory.forEach(r => {
        const log = logs.find(l =>
          Math.abs(Number(l.args.timestamp) - new Date(r.timestamp).getTime() / 1000) < 2 &&
          l.args.sensorId === r.sensorId
        );
        r.txHash = log?.transactionHash || 'Not Found';
      });

      setHistory(fetchedHistory.reverse());

      const latest = fetchedHistory[fetchedHistory.length - 1];
      setTemperature(latest.temperature);
      setHumidity(latest.humidity);
      setDatetime(new Date());
      setError('');
    } catch (err) {
      console.error(err);
      setError("Gagal mengambil data dari blockchain.");
    }
  };

  useEffect(() => {
    fetchData();
    const interval = setInterval(fetchData, 5000); // Auto-refresh every 5 seconds
    return () => clearInterval(interval);
  }, []);

  const cardStyle = {
    backgroundColor: '#fff3e0', padding: '1rem', borderRadius: '12px',
    boxShadow: '0 2px 8px rgba(0,0,0,0.1)', marginBottom: '1rem', color: '#333'
  };
  const navBtnStyle = active => ({
    padding: '0.5rem 1.5rem', borderRadius: '8px', border: 'none',
    cursor: 'pointer', backgroundColor: active ? '#ff9800' : '#fff',
    color: active ? '#fff' : '#000', fontWeight: 'bold'
  });

  const parsedTemp = parseFloat(temperature);
  const parsedHum = parseFloat(humidity);
  const isDataValid = !isNaN(parsedTemp) && !isNaN(parsedHum);
  const isOptimal = parsedTemp >= 30 && parsedTemp <= 35 && parsedHum >= 60 && parsedHum <= 80;
  const isError = parsedTemp < 28 || parsedTemp > 36 || parsedHum < 50 || parsedHum > 90;

  return (
    <div style={{ fontFamily:'Arial,sans-serif', background:'linear-gradient(to right,#000,#ff6f00)',
      width:'100vw', minHeight:'100vh', padding:'2rem', color:'#fff' }}>
      <div>
        <div style={{ display:'flex', justifyContent:'center', gap:'1rem', marginBottom:'2rem' }}>
          {['live','history','pricing'].map(tab => (
            <button key={tab} style={navBtnStyle(view === tab)} onClick={() => setView(tab)}>
              {tab === 'live' ? 'Live Monitor' : tab === 'history' ? 'Transaction History' : 'Harga & Pembelian'}
            </button>
          ))}
        </div>

        <div style={{ maxWidth:'1000px', margin:'auto', backgroundColor:'#ffffffee', padding:'2rem', borderRadius:'16px', boxShadow:'0 8px 20px rgba(0,0,0,0.2)', color:'#000' }}>
          <h1 style={{ textAlign:'center', color:'#d84315', marginBottom:'1.5rem' }}>
            📡 Monitoring Suhu & Kelembapan Tape
          </h1>
          <p><strong>👛 Connected Wallet:</strong> {account || <span style={{color:'red'}}>Not Connected</span>}</p>

          {view === 'live' && (
            <>
              <div style={cardStyle}><p>🌡️ Temperature: {temperature} °C</p></div>
              <div style={cardStyle}><p>💧 Humidity: {humidity} %</p></div>
              <div style={cardStyle}><p>🕒 Time: {datetime.toLocaleString()}</p></div>
            </>
          )}

          {view === 'history' && (
            <>
              <h2>📊 Grafik</h2>
              <ResponsiveContainer width="100%" height={300}>
                <LineChart data={history}>
                  <CartesianGrid strokeDasharray="3 3" />
                  <XAxis dataKey="timestamp" tick={{fontSize:10}}/>
                  <YAxis/>
                  <Tooltip/>
                  <Legend/>
                  <Line type="monotone" dataKey="temperature" stroke="#ff5722" name="Temp (°C)" />
                  <Line type="monotone" dataKey="humidity" stroke="#2196f3" name="Hum (%)" />
                </LineChart>
              </ResponsiveContainer>
              <h2 style={{marginTop:'2rem'}}>📄 Tabel Data</h2>
              <div style={{overflowX:'auto'}}>
                <table style={{width:'100%',borderCollapse:'collapse'}}>
                  <thead><tr style={{background:'#ffe0b2'}}><th>#</th><th>Sensor ID</th><th>Temp</th><th>Hum</th><th>Timestamp</th><th>TX Hash</th></tr></thead>
                  <tbody>
                    {history.map((r,i)=>(
                      <tr key={i}><td>{r.index}</td><td>{r.sensorId}</td><td>{r.temperature}</td><td>{r.humidity}</td><td>{r.timestamp}</td><td style={{fontSize:'10px'}}>{r.txHash}</td></tr>
                    ))}
                  </tbody>
                </table>
              </div>
            </>
          )}

          {view === 'pricing' && (
            <>
              <h2>📦 Harga & Kualitas Tape</h2>
              {isDataValid ? (
                <>
                  <div style={cardStyle}>
                    <p>🌡️ Suhu: {parsedTemp} °C</p>
                    <p>💧 Kelembapan: {parsedHum} %</p>
                    <p>📊 Kualitas: {isOptimal ? "Optimal ✅" : isError ? "Error ⚠️" : "Kurang Optimal ⚠️"}</p>
                    <p>💸 Harga/kg: {
                      (() => {
                        let base = 30000;
                        if (isOptimal) return "Rp " + (base + 5000).toLocaleString();
                        if (isError) return "Rp " + (base - 5000).toLocaleString();
                        return "Rp " + base.toLocaleString();
                      })()
                    }</p>
                  </div>
                  <h2 style={{ marginTop:'2rem' }}>🛒 QR Code Pembayaran</h2>
                  <div style={{
                    background:'#fefefe',padding:'1rem',display:'flex',flexDirection:'column',
                    alignItems:'center',borderRadius:'12px',boxShadow:'0 4px 8px rgba(0,0,0,0.1)'
                  }}>
                    <p><strong>Masukkan Alamat Wallet (atau gunakan koneksi MetaMask):</strong></p>
                    <input
                      type="text"
                      placeholder="0x..."
                      value={account}
                      onChange={e => setAccount(e.target.value)}
                      style={{padding:'0.5rem',width:'100%',marginBottom:'1rem',border:'1px solid #ccc',borderRadius:'6px'}}
                    />
                    <QRCode value={account || "0x005B6cCDdDb9119b2F94F839019Cc1ED3CcB06c3"} size={180} />
                    <p style={{fontSize:'12px', marginTop:'1rem', wordBreak:'break-all'}}>
                      {account || "Wallet belum diisi"}
                    </p>
                  </div>
                </>
              ) : (
                <p style={{color:'gray'}}>❗ Data suhu & kelembapan belum tersedia untuk kalkulasi harga.</p>
              )}
            </>
          )}

          {error && <p style={{color:'red', marginTop:'1rem'}}>⚠️ {error}</p>}
        </div>
      </div>
    </div>
  );
}

export default App;
