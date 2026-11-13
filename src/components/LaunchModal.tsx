export default function LaunchModal({ status, progress, error, onClose }: { 
  status: string, 
  progress: number,
  error: string | null, 
  onClose: () => void 
}) {
  return (
    <div className="modal-overlay" style={{ zIndex: 1000 }}>
      <div className="modal-content" style={{ width: 450, textAlign: 'center', padding: '40px 24px' }} onClick={e => e.stopPropagation()}>
        
        <h2 style={{ marginTop: 0, color: error ? 'var(--danger)' : 'var(--primary)', marginBottom: 20 }}>
          {error ? 'Ошибка запуска' : 'Запуск Minecraft'}
        </h2>
        
        {error ? (
          <>
            <div style={{ color: 'var(--danger)', background: 'rgba(239, 68, 68, 0.1)', padding: 16, borderRadius: 8, textAlign: 'left', fontSize: 14, marginBottom: 20 }}>
              {error}
            </div>
            <button onClick={onClose} style={{ width: '100%' }}>
              Закрыть
            </button>
          </>
        ) : (
          <>
            <p style={{ color: 'var(--text-muted)', marginBottom: 20 }}>{status}</p>
            
            <div className="progress-container" style={{ maxWidth: '100%', marginBottom: 20 }}>
              <div style={{ display: 'flex', justifyContent: 'space-between', fontSize: 12, color: 'var(--text-muted)', marginBottom: 8 }}>
                <span>Прогресс</span>
                <span>{progress}%</span>
              </div>
              <div className="progress-bar-bg">
                <div className="progress-bar-fill" style={{ width: `${progress}%`, background: 'var(--primary)' }} />
              </div>
            </div>
          </>
        )}
      </div>
    </div>
  );
}