// src/App.jsx
import { useState } from 'react';
import { invoke } from '@tauri-apps/api/core';

function App() {
  const [username, setUsername] = useState('alice');
  const [password, setPassword] = useState('password');
  const [user, setUser] = useState(null);
  const [error, setError] = useState('');

  const handleLogin = async () => {
    try {
      const result = await invoke('login', { username, password });
      setUser(result);
      setError('');
    } catch (e) {
      setError(e);
    }
  };

  const handleLaunch = async () => {
    try {
      await invoke('launch_game', { version: '1.20.4' });
    } catch (e) {
      setError(e);
    }
  };

  return (
    <div style={{ backgroundColor: '#0f172a', color: 'white', height: '100vh', display: 'flex', flexDirection: 'column', alignItems: 'center', justifyContent: 'center' }}>
      <h1>Sporelink Launcher</h1>
      {!user ? (
        <>
          <input value={username} onChange={e => setUsername(e.target.value)} placeholder="Username" />
          <input type="password" value={password} onChange={e => setPassword(e.target.value)} placeholder="Password" />
          <button onClick={handleLogin}>Log In</button>
        </>
      ) : (
        <>
          <p>Welcome, {user.name}</p>
          <button onClick={handleLaunch}>▶ PLAY</button>
        </>
      )}
      {error && <p style={{ color: 'red' }}>{error}</p>}
    </div>
  );
}

export default App;