import { useState, useEffect } from 'react';
import { invoke } from '@tauri-apps/api/core';
import SkinPreview from './SkinPreview';
import './App.css';

function App() {
  const [username, setUsername] = useState('alice');
  const [password, setPassword] = useState('password');
  const [user, setUser] = useState<{name: string, uuid: string} | null>(null);
  
  const [skinData, setSkinData] = useState<string | null>(null);
  const [ram, setRam] = useState(2048);
  
  const [error, setError] = useState('');
  const [loading, setLoading] = useState(false);

  const handleLogin = async () => {
    setLoading(true);
    try {
      const result: any = await invoke('login', { username, password });
      setUser({ name: result.name, uuid: result.uuid });
      setError('');
      
      // Загружаем скин
      try {
        const skin = await invoke<string>('get_skin_data', { uuid: result.uuid });
        setSkinData(skin);
      } catch (e) {
        console.log("No skin found or error loading it");
      }
      
    } catch (e) {
      setError(e as string);
    } finally {
      setLoading(false);
    }
  };

  const handleLaunch = async () => {
    setLoading(true);
    try {
      await invoke('launch_game', { version: '1.20.4', ramMb: ram });
    } catch (e) {
      setError(e as string);
    } finally {
      setLoading(false);
    }
  };

  return (
    <div className="launcher-container">
      <h1>Sporelink</h1>
      
      {!user ? (
        <>
          <input value={username} onChange={e => setUsername(e.target.value)} placeholder="Username" disabled={loading} />
          <input type="password" value={password} onChange={e => setPassword(e.target.value)} placeholder="Password" disabled={loading} />
          <button onClick={handleLogin} disabled={loading}>
            {loading ? 'Logging in...' : 'Log In'}
          </button>
        </>
      ) : (
        <div className="logged-in-view">
          <div className="user-info" style={{ paddingBottom: 10 }}>
            <strong>{user.name}</strong>
            <span>{user.uuid}</span>
          </div>

          <SkinPreview skinUrl={skinData} />

          <div style={{ marginTop: 20, textAlign: 'left', width: '100%' }}>
            <label style={{ fontSize: 13, color: 'var(--text-muted)' }}>
              Выделить память (RAM): <b>{ram} MB</b>
            </label>
            <input 
              type="range" 
              min="1024" 
              max="8192" 
              step="512" 
              value={ram} 
              onChange={e => setRam(Number(e.target.value))}
              style={{ width: '100%', accentColor: 'var(--primary)', marginTop: 5 }}
            />
          </div>

          <button onClick={handleLaunch} disabled={loading} style={{ marginTop: 20 }}>
            {loading ? 'Launching...' : '▶ PLAY'}
          </button>
        </div>
      )}

      {error && <div className="error-msg">{error}</div>}
    </div>
  );
}

export default App;