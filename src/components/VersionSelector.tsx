import { useState } from 'react';

export default function VersionSelector({ versions, selected, onChange }: { versions: string[], selected: string, onChange: (v: string) => void }) {
  const [isOpen, setIsOpen] = useState(false);

  return (
    <div style={{ position: 'relative', width: '250px', marginBottom: 20, zIndex: 100 }}>
      {/* Кнопка, которая открывает список */}
      <div 
        onClick={() => setIsOpen(!isOpen)}
        style={{
          padding: '12px 16px',
          background: 'var(--panel)',
          border: '1px solid var(--border)',
          borderRadius: 8,
          cursor: 'pointer',
          display: 'flex',
          justifyContent: 'space-between',
          alignItems: 'center',
          color: 'var(--text)',
          fontSize: 14,
          userSelect: 'none',
        }}
      >
        {selected || 'Выберите версию'}
        <span style={{ fontSize: 10, color: 'var(--text-muted)' }}>{isOpen ? '▲' : '▼'}</span>
      </div>

      {/* Сам выпадающий список */}
      {isOpen && (
        <div style={{
          position: 'absolute',
          top: '100%',
          left: 0,
          right: 0,
          background: 'var(--panel)',
          border: '1px solid var(--border)',
          borderRadius: 8,
          marginTop: 4,
          maxHeight: '200px',
          overflowY: 'auto',
          boxShadow: '0 10px 20px rgba(0,0,0,0.5)',
          zIndex: 100
        }}>
          {versions.length === 0 ? (
            <div style={{ padding: '12px', color: 'var(--text-muted)', fontSize: 13, textAlign: 'center' }}>
              Загрузка версий...
            </div>
          ) : (
            versions.map(v => (
              <div
                key={v}
                onClick={() => { onChange(v); setIsOpen(false); }}
                style={{
                    padding: '10px 16px',
                    cursor: 'pointer',
                    borderBottom: '1px solid var(--border)',
                    color: v === selected ? 'var(--primary)' : 'var(--text)',
                    background: v === selected ? 'rgba(59, 130, 246, 0.1)' : 'transparent',
                    fontSize: 14,
                    transition: 'background 0.1s'
                }}
                onMouseEnter={(e) => (e.currentTarget.style.background = 'var(--panel-hover)')}
                onMouseLeave={(e) => (e.currentTarget.style.background = v === selected ? 'rgba(59, 130, 246, 0.1)' : 'transparent')}
              >
                {v}
              </div>
            ))
          )}
        </div>
      )}
    </div>
  );
}