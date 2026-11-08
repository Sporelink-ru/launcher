import { useState, useEffect } from 'react';
import { invoke } from '@tauri-apps/api/core';
import { listen } from '@tauri-apps/api/event';
import { openUrl } from '@tauri-apps/plugin-opener';
import TitleBar from './components/TitleBar';
import UserModal from './components/UserModal';
import LaunchModal from './components/LaunchModal';
import SettingsModal from './components/SettingsModal';
import './App.css';

export default function App() {
  const [username, setUsername] = useState('alice');
  const [password, setPassword] = useState('password');
  const [user, setUser] = useState<{name: string, uuid: string} | null>(null);
  const [skinData, setSkinData] = useState<string | null>(null);
  const [error, setError] = useState('');
  const [loading, setLoading] = useState(false);

  const [isUserModalOpen, setIsUserModalOpen] = useState(false);
  const [isSettingsOpen, setIsSettingsOpen] = useState(false);
  const [ram, setRam] = useState(2048);

  const [isLaunching, setIsLaunching] = useState(false);
  const [launchStatus, setLaunchStatus] = useState('');
  const [launchLogs, setLaunchLogs] = useState<string[]>([]);
  const [launchError, setLaunchError] = useState<string | null>(null);

  // 1. Проверка сохраненной сессии при старте
  useEffect(() => {
    invoke<any>('get_saved_session').then(session => {
      if (session) {
        setUser({ name: session.name, uuid: session.uuid });
      }
    }).catch(console.error);
  }, []);

  // 2. Загрузка скина при появлении пользователя
  useEffect(() => {
    if (user) {
      invoke<string>('get_skin_data', { uuid: user.uuid })
        .then(skin => setSkinData(skin))
        .catch(e => {
          console.error("Skin error:", e);
          setSkinData(null);
        });
    }
  }, [user]);

  // 3. Подписка на события из Rust
  useEffect(() => {
    let unlistenStatus: any;
    let unlistenLog: any;
    let unlistenClosed: any;

    async function setupListeners() {
      unlistenStatus = await listen<string>('launch_status', (event) => {
        setLaunchStatus(event.payload);
      });
      unlistenLog = await listen<string>('launch_log', (event) => {
        setLaunchLogs(prev => [...prev.slice(-50), event.payload]);
      });
      unlistenClosed = await listen('game_closed', () => {
        setIsLaunching(false);
        setLaunchLogs([]);
        setLaunchStatus('');
      });
    }
    setupListeners();

    return () => {
      if (unlistenStatus) unlistenStatus();
      if (unlistenLog) unlistenLog();
      if (unlistenClosed) unlistenClosed();
    };
  }, []);

  const handleLogin = async () => {
    setLoading(true);
    setError('');
    try {
      const result: any = await invoke('login', { username, password });
      setUser({ name: result.name, uuid: result.uuid });
    } catch (e) {
      setError(e as string);
    } finally {
      setLoading(false);
    }
  };

  const handleLaunch = async () => {
    setIsLaunching(true);
    setLaunchError(null);
    setLaunchStatus('Инициализация...');
    setLaunchLogs([]);
    
    try {
      await invoke('launch_game', { version: '1.20.4', ramMb: ram });
    } catch (e) {
      setLaunchError(e as string);
    }
  };

  const handleLogout = () => {
    invoke('logout_cmd').catch(() => {});
    setUser(null);
    setSkinData(null);
    setPassword('');
  };

  return (
    <div className="app-container">
      <TitleBar 
        user={user} 
        skinData={skinData}
        onOpenUserModal={() => setIsUserModalOpen(true)} 
        onOpenSettings={() => setIsSettingsOpen(true)}
        onLogout={handleLogout} 
      />

      <div className="main-content">
        {!user ? (
          <div key="login" className="login-box animate-in">
            <h2 style={{ marginTop: 0 }}>Вход</h2>
            <input value={username} onChange={e => setUsername(e.target.value)} placeholder="Username" disabled={loading} />
            <input type="password" value={password} onChange={e => setPassword(e.target.value)} placeholder="Password" disabled={loading} />
            <button onClick={handleLogin} disabled={loading}>
              {loading ? 'Загрузка...' : 'Войти'}
            </button>
            {error && <div className="error-msg">{error}</div>}

            <div className="auth-links">
              <span className="link" onClick={() => openUrl('https://sporelink.ru')}>
                Регистрация
              </span>
              <span className="divider">•</span>
              <span className="link" onClick={() => openUrl('https://sporelink.ru')}>
                Забыли пароль?
              </span>
            </div>
          </div>
        ) : (
          <div key="dashboard" className="animate-in" style={{ textAlign: 'center', width: '100%' }}>
            <h1 style={{ fontSize: 36, marginBottom: 10, textShadow: '0 4px 20px rgba(0,0,0,0.5)' }}>
              Добро пожаловать, <span style={{ color: 'var(--primary)' }}>{user.name}</span>
            </h1>
            <p style={{ color: 'var(--text-muted)', marginBottom: 40 }}>С возвращением в Sporelink Network</p>

            <button className="play-btn" onClick={handleLaunch} disabled={loading || isLaunching}>
              {isLaunching ? 'Игра запущена...' : '▶ ИГРАТЬ'}
            </button>
          </div>
        )}
      </div>

      {isUserModalOpen && user && (
        <UserModal skinData={skinData} onClose={() => setIsUserModalOpen(false)} />
      )}
      
      {isLaunching && (
        <LaunchModal 
          status={launchStatus} 
          logs={launchLogs} 
          error={launchError} 
        />
      )}

      {isSettingsOpen && (
        <SettingsModal ram={ram} setRam={setRam} onClose={() => setIsSettingsOpen(false)} />
      )}
    </div>
  );
}