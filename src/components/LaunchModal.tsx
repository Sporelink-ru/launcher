export default function LaunchModal({ status, logs, error }: { status: string, logs: string[], error: string | null }) {
  return (
    <div className="modal-overlay" style={{ zIndex: 1000 }}>
      <div className="modal-content" style={{ width: 500, textAlign: 'center' }} onClick={e => e.stopPropagation()}>
        <h2 style={{ marginTop: 0, color: 'var(--primary)' }}>
          {error ? 'Ошибка запуска' : 'Запуск Minecraft'}
        </h2>
        
        {error ? (
          <div style={{ color: 'var(--danger)', background: 'rgba(239, 68, 68, 0.1)', padding: 12, borderRadius: 8, textAlign: 'left', fontSize: 14 }}>
            {error}
          </div>
        ) : (
          <>
            <p style={{ color: 'var(--text-muted)', marginBottom: 20 }}>{status}</p>
            
            {/* Бесконечный прогресс (так как мы не знаем точное время загрузки Java) */}
            <div className="progress-container" style={{ maxWidth: '100%' }}>
              <div className="progress-bar-bg">
                <div className="progress-bar-fill" style={{ width: '100%', animation: 'pulse 1.5s infinite' }} />
              </div>
            </div>

            <div style={{ 
              marginTop: 24, 
              background: '#0f172a', 
              padding: 12, 
              borderRadius: 8, 
              fontFamily: 'monospace', 
              fontSize: 12, 
              color: '#10b981',
              textAlign: 'left',
              height: 120,
              overflowY: 'auto',
              display: 'flex',
              flexDirection: 'column',
              gap: 4
            }}>
              {logs.length === 0 ? <div>Ожидание логов...</div> : null}
              {logs.map((log, i) => (
                <div key={i} style={{ wordBreak: 'break-all' }}>{log}</div>
              ))}
            </div>
          </>
        )}
      </div>
    </div>
  );
}