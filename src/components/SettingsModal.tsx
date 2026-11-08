export default function SettingsModal({ ram, setRam, onClose }: { ram: number, setRam: (v: number) => void, onClose: () => void }) {
  return (
    <div className="modal-overlay" onClick={onClose}>
      <div className="modal-content" onClick={e => e.stopPropagation()} style={{ width: 400 }}>
        <button className="modal-close" onClick={onClose}>✖</button>
        <h2 style={{ marginTop: 0, textAlign: 'center' }}>Настройки</h2>
        
        <div style={{ marginTop: 24 }}>
          <label style={{ fontSize: 14, color: 'var(--text-muted)', display: 'flex', justifyContent: 'space-between' }}>
            <span>Выделить память (RAM)</span>
            <span style={{ color: 'var(--primary)', fontWeight: 'bold' }}>{ram} MB</span>
          </label>
          <input 
            type="range" 
            min="1024" 
            max="8192" 
            step="512" 
            value={ram} 
            onChange={e => setRam(Number(e.target.value))}
            style={{ width: '100%', accentColor: 'var(--primary)', marginTop: 12 }}
          />
          <p style={{ fontSize: 12, color: 'var(--text-muted)', marginTop: 8 }}>
            Рекомендуется выделять не более 50% от объема вашей оперативной памяти. Для сборок с модами может потребоваться 4-6 ГБ.
          </p>
        </div>

        <button onClick={onClose} style={{ marginTop: 24 }}>
          Сохранить и закрыть
        </button>
      </div>
    </div>
  );
}