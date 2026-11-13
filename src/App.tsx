import { useState, useEffect } from 'react';
import { invoke } from '@tauri-apps/api/core';
import { listen } from '@tauri-apps/api/event';
import TitleBar from './components/TitleBar';
import UserModal from './components/UserModal';
import LaunchModal from './components/LaunchModal';
import SettingsModal from './components/SettingsModal';
import NewsModal from './components/NewsModal';
import VersionSelector from './components/VersionSelector';
import './App.css';

export default function App() {
  // --- Авторизация ---
  const [username, setUsername] = useState('alice');
  const [password, setPassword] = useState('password');
  const [user, setUser] = useState<{name: string, uuid: string} | null>(null);
  const [textures, setTextures] = useState<{skin: string | null, cape: string | null}>({skin: null, cape: null});
  const [error, setError] = useState('');
  const [loading, setLoading] = useState(false);

  // --- Настройки и модалки ---
  const [isUserModalOpen, setIsUserModalOpen] = useState(false);
  const [isSettingsOpen, setIsSettingsOpen] = useState(false);
  const [isNewsOpen, setIsNewsOpen] = useState(false);
  const [ram, setRam] = useState(2048);

  // --- Версии и запуск ---
  const [versions, setVersions] = useState<string[]>([]);
  const [selectedVersion, setSelectedVersion] = useState("1.20.4");
  const [isLaunching, setIsLaunching] = useState(false);
  const [launchStatus, setLaunchStatus] = useState('');
  const [launchProgress, setLaunchProgress] = useState(0); // <--- ВОТ ОНО!
  const [launchError, setLaunchError] = useState<string | null>(null);

  // 1. Проверка сохраненной сессии
  useEffect(() => {
    invoke<any>('get_saved_session').then(session => {
      if (session) setUser({ name: session.name, uuid: session.uuid });
    }).catch(console.error);
  }, []);

  // 2. Загрузка скинов
  const refreshTextures = () => {
    if (user) {
      invoke<any>('get_textures_data', { uuid: user.uuid })
        .then(data => setTextures({ skin: data.skin, cape: data.cape }))
        .catch(e => console.error(e));
    }
  };

  useEffect(() => { refreshTextures(); }, [user]);

  // 3. Загрузка списка версий
  useEffect(() => {
    invoke<string[]>('get_minecraft_versions')
      .then(v => {
        setVersions(v);
        if (!v.includes(selectedVersion) && v.length > 0) setSelectedVersion(v[0]);
      })
      .catch(err => console.error("Error fetching versions:", err));
  }, []);

  // 4. Подписка на события из Rust (логи, статусы, закрытие игры)
  useEffect(() => {
    let unlistenStatus: any, unlistenLog: any, unlistenClosed: any;

    async function setupListeners() {
      unlistenStatus = await listen<string>('launch_status', (event) => setLaunchStatus(event.payload));
      unlistenClosed = await listen('game_closed', () => {
        setLaunchStatus('Игра успешно закрыта.');
      });
    }
    setupListeners();

    return () => {
      if (unlistenStatus) unlistenStatus();
      if (unlistenLog) unlistenLog();
      if (unlistenClosed) unlistenClosed();
    };
  }, []);

  // 5. Слушатель события launch_progress из Rust
  useEffect(() => {
    let unlistenStatus: any, unlistenClosed: any, unlistenProgress: any;

    async function setupListeners() {
      unlistenStatus = await listen<string>('launch_status', (event) => setLaunchStatus(event.payload));
      unlistenProgress = await listen<number>('launch_progress', (event) => {
        setLaunchProgress(event.payload);
      });
    }
    setupListeners();

    return () => {
      if (unlistenStatus) unlistenStatus();
      if (unlistenClosed) unlistenClosed();
      if (unlistenProgress) unlistenProgress();
    };
  }, []);

  const handleLogin = async () => {
    setLoading(true); setError('');
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
    setLaunchStatus('Подготовка...');
    setLaunchProgress(0);

    try {
      await invoke('download_game_and_java', { version: selectedVersion });
      
      setLaunchStatus('Запуск Minecraft...');
      setLaunchProgress(100);
      
      await invoke('launch_game', { version: selectedVersion, ramMb: ram });
      
      setLaunchStatus('Успешно! Игра запускается...');
      setTimeout(() => {
        setIsLaunching(false);
        setLaunchProgress(0);
        setLaunchStatus('');
      }, 3000);

    } catch (e) {
      setLaunchError(e as string);
      setLaunchStatus('Ошибка');
    }
  };

  const closeLaunchModal = () => {
    setIsLaunching(false);
    setLaunchStatus('');
    setLaunchProgress(0);
  };

  const handleLogout = () => {
    setUser(null); setTextures({skin: null, cape: null}); setPassword(''); setUsername('');
  };

  return (
    <div className="app-container">
      <TitleBar 
        user={user} 
        skinData={textures.skin}
        onOpenUserModal={() => setIsUserModalOpen(true)} 
        onOpenSettings={() => setIsSettingsOpen(true)}
        onOpenNews={() => setIsNewsOpen(true)}
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
          </div>
        ) : (
          <div key="dashboard" className="animate-in" style={{ textAlign: 'center', width: '100%', display: 'flex', flexDirection: 'column', alignItems: 'center' }}>
            <h1 style={{ fontSize: 36, marginBottom: 10, textShadow: '0 4px 20px rgba(0,0,0,0.5)', marginTop: 40 }}>
              Добро пожаловать, <span style={{ color: 'var(--primary)' }}>{user.name}</span>
            </h1>
            <p style={{ color: 'var(--text-muted)', marginBottom: 40 }}>С возвращением в Sporelink Network</p>

            <div style={{ display: 'flex', flexDirection: 'column', alignItems: 'center', gap: 4 }}>
              <label style={{ fontSize: 13, color: 'var(--text-muted)' }}>Выберите версию Minecraft</label>
              <VersionSelector 
                versions={versions} 
                selected={selectedVersion} 
                onChange={setSelectedVersion} 
              />
            </div>

            <button className="play-btn" onClick={handleLaunch} disabled={loading || isLaunching} style={{ width: 250 }}>
              {isLaunching ? 'Запуск...' : '▶ ИГРАТЬ'}
            </button>
            
            {error && <div className="error-msg">{error}</div>}
          </div>
        )}
      </div>

      {/* Модалки */}
      {isUserModalOpen && user && (
        <UserModal user={user} textures={textures} onClose={() => setIsUserModalOpen(false)} onRefreshSkin={refreshTextures} />
      )}
      
      {isLaunching && (
        <LaunchModal 
          progress={launchProgress}
          status={launchStatus} 
          error={launchError} 
          onClose={closeLaunchModal}
        />
      )}

      {isSettingsOpen && (
        <SettingsModal ram={ram} setRam={setRam} onClose={() => setIsSettingsOpen(false)} />
      )}

      {isNewsOpen && (
        <NewsModal onClose={() => setIsNewsOpen(false)} />
      )}
    </div>
  );
}