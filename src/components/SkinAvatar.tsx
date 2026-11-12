import { useState, useEffect } from 'react';

export default function SkinAvatar({ skinData, fallbackText, size = 32 }: { skinData: string | null, fallbackText: string, size?: number }) {
  const [blobUrl, setBlobUrl] = useState<string | null>(null);

  useEffect(() => {
    if (!skinData) {
      setBlobUrl(null);
      return;
    }
    try {
      // Конвертируем base64 в Blob URL (обходит ограничения безопасности Tauri)
      const parts = skinData.split(',');
      const mime = parts[0].match(/:(.*?);/)![1];
      const bstr = atob(parts[1]);
      let n = bstr.length;
      const u8arr = new Uint8Array(n);
      while (n--) {
        u8arr[n] = bstr.charCodeAt(n);
      }
      const url = URL.createObjectURL(new Blob([u8arr], { type: mime }));
      setBlobUrl(url);
      
      return () => URL.revokeObjectURL(url); // Очистка памяти
    } catch (e) {
      console.error("Failed to parse skin data", e);
      setBlobUrl(null);
    }
  }, [skinData]);

  if (!blobUrl) {
    return (
      <div style={{ 
        width: size, height: size, flexShrink: 0, 
        background: 'var(--primary)', borderRadius: 6,
        display: 'flex', alignItems: 'center', justifyContent: 'center', fontWeight: 'bold' 
      }}>
        {fallbackText[0].toUpperCase()}
      </div>
    );
  }

  return (
    <div style={{
      width: size, height: size, flexShrink: 0,
      borderRadius: 6, overflow: 'hidden', 
      background: 'var(--bg)', border: '1px solid var(--border)', 
      position: 'relative'
    }}>
      {/* Лицо (вырезаем 8x8 пикселей) */}
      <img 
        src={blobUrl} 
        style={{
          position: 'absolute',
          width: `${size * 8}px`,
          height: `${size * 8}px`,
          top: `-${size}px`,
          left: `-${size}px`,
          imageRendering: 'pixelated'
        }} 
      />
      {/* Слой шляпы (вырезаем 8x8 пикселей со сдвигом 32px) */}
      <img 
        src={blobUrl} 
        style={{
          position: 'absolute',
          width: `${size * 8}px`,
          height: `${size * 8}px`,
          top: `-${size}px`,
          left: `-${size * 5}px`,
          imageRendering: 'pixelated'
        }} 
      />
    </div>
  );
}