import { useEffect, useState } from 'react';
import { invoke } from '@tauri-apps/api/core';

type NewsItem = { id: number, title: string, content: string, date: string };

export default function NewsModal({ onClose }: { onClose: () => void }) {
  const [news, setNews] = useState<NewsItem[]>([]);
  const [loading, setLoading] = useState(true);
  const [error, setError] = useState<string | null>(null);

  useEffect(() => {
    invoke<NewsItem[]>('get_news')
      .then(data => { setNews(data); setLoading(false); })
      .catch(e => { setError(e as string); setLoading(false); });
  }, []);

  return (
    <div className="modal-overlay" onClick={onClose}>
      <div className="modal-content" onClick={e => e.stopPropagation()} style={{ width: 500 }}>
        <button className="modal-close" onClick={onClose}>✖</button>
        <h2 style={{ marginTop: 0, textAlign: 'center', color: 'var(--primary)' }}>Новости сети</h2>
        
        <div style={{ maxHeight: 400, overflowY: 'auto', marginTop: 20 }}>
          {loading && <p style={{ textAlign: 'center', color: 'var(--text-muted)' }}>Загрузка...</p>}
          {error && <p style={{ color: 'var(--danger)' }}>{error}</p>}
          
          {news.map(item => (
            <div key={item.id} style={{ 
              background: 'var(--bg)', padding: 16, borderRadius: 8, marginBottom: 12, 
              borderLeft: '4px solid var(--primary)' 
            }}>
              <div style={{ display: 'flex', justifyContent: 'space-between', marginBottom: 8 }}>
                <strong style={{ fontSize: 16 }}>{item.title}</strong>
                <span style={{ fontSize: 12, color: 'var(--text-muted)' }}>{item.date}</span>
              </div>
              <p style={{ margin: 0, fontSize: 14, lineHeight: 1.5, color: 'var(--text-muted)' }}>
                {item.content}
              </p>
            </div>
          ))}
        </div>
      </div>
    </div>
  );
}