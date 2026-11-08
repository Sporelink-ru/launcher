import { getCurrentWindow } from '@tauri-apps/api/window';
import { openUrl } from '@tauri-apps/plugin-opener';
import SkinAvatar from './SkinAvatar';

const appWindow = getCurrentWindow();

export default function TitleBar({ user, skinData, onOpenUserModal, onOpenSettings, onLogout }: any) {
  return (
    <div className="title-bar" data-tauri-drag-region>
      
      <div className="title-bar-left">
        {user && (
          <div className="user-profile-trigger" onClick={onOpenUserModal}>
            <SkinAvatar skinData={skinData} fallbackText={user.name} size={32} />
            <span style={{ fontSize: 14, fontWeight: 500 }}>{user.name}</span>
          </div>
        )}
      </div>

      <div 
        className="title-bar-center" 
        data-tauri-drag-region 
        style={{ flex: 1, textAlign: 'center', height: '100%', display: 'flex', alignItems: 'center', justifyContent: 'center' }}
      >
        <a onClick={(e) => { 
          e.preventDefault(); 
          openUrl('https://sporelink.ru'); 
        }}>
          SPORELINK
        </a>
      </div>

      <div className="title-bar-right">
        {user && (
          <>
            <button className="win-btn" title="Настройки" onClick={onOpenSettings}>
              <svg width="18" height="18" viewBox="0 0 24 24" fill="none" stroke="currentColor" strokeWidth="2" strokeLinecap="round" strokeLinejoin="round">
                <circle cx="12" cy="12" r="3"></circle>
                <path d="M19.4 15a1.65 1.65 0 0 0 .33 1.82l.06.06a2 2 0 0 1 0 2.83 2 2 0 0 1-2.83 0l-.06-.06a1.65 1.65 0 0 0-1.82-.33 1.65 1.65 0 0 0-1 1.51V21a2 2 0 0 1-2 2 2 2 0 0 1-2-2v-.09A1.65 1.65 0 0 0 9 19.4a1.65 1.65 0 0 0-1.82.33l-.06.06a2 2 0 0 1-2.83 0 2 2 0 0 1 0-2.83l.06-.06a1.65 1.65 0 0 0 .33-1.82 1.65 1.65 0 0 0-1.51-1H3a2 2 0 0 1-2-2 2 2 0 0 1 2-2h.09A1.65 1.65 0 0 0 4.6 9a1.65 1.65 0 0 0-.33-1.82l-.06-.06a2 2 0 0 1 0-2.83 2 2 0 0 1 2.83 0l.06.06a1.65 1.65 0 0 0 1.82.33H9a1.65 1.65 0 0 0 1-1.51V3a2 2 0 0 1 2-2 2 2 0 0 1 2 2v.09a1.65 1.65 0 0 0 1 1.51 1.65 1.65 0 0 0 1.82-.33l.06-.06a2 2 0 0 1 2.83 0 2 2 0 0 1 0 2.83l-.06.06a1.65 1.65 0 0 0-.33 1.82V9a1.65 1.65 0 0 0 1.51 1H21a2 2 0 0 1 2 2 2 2 0 0 1-2 2h-.09a1.65 1.65 0 0 0-1.51 1z"></path>
              </svg>
            </button>
            <button className="win-btn" title="Выйти" onClick={onLogout}>
              <svg width="18" height="18" viewBox="0 0 24 24" fill="none" stroke="currentColor" strokeWidth="2" strokeLinecap="round" strokeLinejoin="round">
                <path d="M9 21H5a2 2 0 0 1-2-2V5a2 2 0 0 1 2-2h4"></path>
                <polyline points="16 17 21 12 16 7"></polyline>
                <line x1="21" y1="12" x2="9" y2="12"></line>
              </svg>
            </button>
          </>
        )}
        
        <div style={{ width: 1, height: 20, background: 'var(--border)', margin: '0 8px' }}></div>

        <div className="window-controls">
          <button className="win-btn" onClick={() => appWindow.minimize()} title="Свернуть">
            <svg width="12" height="12" viewBox="0 0 12 12"><rect y="5" width="12" height="1" fill="currentColor"/></svg>
          </button>
          <button className="win-btn" onClick={() => appWindow.toggleMaximize()} title="Развернуть">
            <svg width="12" height="12" viewBox="0 0 12 12"><rect x="1.5" y="1.5" width="9" height="9" fill="none" stroke="currentColor" strokeWidth="1.5"/></svg>
          </button>
          <button className="win-btn close" onClick={() => appWindow.close()} title="Закрыть">
            <svg width="12" height="12" viewBox="0 0 12 12"><path d="M1.5 1.5l9 9m-9 0l9-9" stroke="currentColor" strokeWidth="1.5"/></svg>
          </button>
        </div>
      </div>
    </div>
  );
}