import { useState } from 'react';
import { invoke } from '@tauri-apps/api/core';
import SkinPreview from '../SkinPreview';

export default function UserModal({ user, textures, onClose, onRefreshSkin }: any) {
  const [uploading, setUploading] = useState(false);
  const [model, setModel] = useState<'slim' | 'classic'>('slim');
  const [error, setError] = useState<string | null>(null);

  const handleUpload = async (e: React.ChangeEvent<HTMLInputElement>, type: 'skin' | 'cape') => {
    const file = e.target.files?.[0];
    if (!file) return;
    setUploading(true); setError(null);
    try {
      const buffer = await file.arrayBuffer();
      await invoke('upload_texture', { 
        uuid: user.uuid, fileBytes: Array.from(new Uint8Array(buffer)), model, textureType: type 
      });
      onRefreshSkin();
    } catch (err) { setError(err as string); } finally { setUploading(false); }
  };

  return (
    <div className="modal-overlay" onClick={onClose}>
      <div className="modal-content" onClick={e => e.stopPropagation()}>
        <button className="modal-close" onClick={onClose}>✖</button>
        <h2 style={{ marginTop: 0, textAlign: 'center' }}>Профиль</h2>
        
        <div style={{ display: 'flex', flexDirection: 'column', alignItems: 'center' }}>
          <SkinPreview skinUrl={textures.skin} capeUrl={textures.cape} />
          
          <div style={{ width: '100%', marginTop: 20 }}>
            <h3 style={{ fontSize: 14, color: 'var(--text-muted)', marginBottom: 8 }}>Сменить скин</h3>
            <div style={{ display: 'flex', gap: 20, marginBottom: 12 }}>
              <label className="custom-radio"><input type="radio" checked={model === 'slim'} onChange={() => setModel('slim')} /><span className="checkmark"></span><span>Slim</span></label>
              <label className="custom-radio"><input type="radio" checked={model === 'classic'} onChange={() => setModel('classic')} /><span className="checkmark"></span><span>Classic</span></label>
            </div>
            <input type="file" accept="image/png" onChange={(e) => handleUpload(e, 'skin')} disabled={uploading} style={{ padding: 8, marginBottom: 16 }} />

            <h3 style={{ fontSize: 14, color: 'var(--text-muted)', marginBottom: 8 }}>Сменить плащ (Cape)</h3>
            <input type="file" accept="image/png" onChange={(e) => handleUpload(e, 'cape')} disabled={uploading} style={{ padding: 8 }} />

            {uploading && <p style={{ color: 'var(--primary)', fontSize: 13 }}>Загрузка...</p>}
            {error && <p style={{ color: 'var(--danger)', fontSize: 13 }}>{error}</p>}
          </div>
        </div>
      </div>
    </div>
  );
}