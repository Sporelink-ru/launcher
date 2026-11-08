import SkinPreview from '../SkinPreview'; // Ваш существующий компонент

export default function UserModal({ skinData, onClose }: any) {
  return (
    <div className="modal-overlay" onClick={onClose}>
      <div className="modal-content" onClick={e => e.stopPropagation()}>
        <button className="modal-close" onClick={onClose}>
            <svg width="12" height="12" viewBox="0 0 12 12"><path d="M1.5 1.5l9 9m-9 0l9-9" stroke="currentColor" strokeWidth="1.5"/></svg>
        </button>
        <h2 style={{ marginTop: 0, textAlign: 'center' }}>Профиль</h2>
        
        <div style={{ display: 'flex', flexDirection: 'column', alignItems: 'center' }}>
          <SkinPreview skinUrl={skinData} />
          
          <div style={{ width: '100%', marginTop: 20 }}>
            <h3 style={{ fontSize: 14, color: 'var(--text-muted)' }}>Сменить скин</h3>
            <input type="file" accept="image/png" style={{ background: 'var(--bg)', padding: 8 }} />
            <button style={{ marginTop: 8 }}>Загрузить</button>
          </div>
        </div>
      </div>
    </div>
  );
}