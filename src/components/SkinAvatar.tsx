export default function SkinAvatar({ skinData, fallbackText, size = 32 }: { skinData: string | null, fallbackText: string, size?: number }) {
  if (!skinData) {
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

  const scale = size / 8;

  return (
    <div style={{ 
      width: size, height: size, flexShrink: 0,
      borderRadius: 6, overflow: 'hidden', 
      background: 'var(--bg)', border: '1px solid var(--border)', 
      position: 'relative'
    }}>
      {/* Лицо */}
      <div style={{
        position: 'absolute', top: 0, left: 0, width: '100%', height: '100%',
        backgroundImage: `url(${skinData})`,
        backgroundSize: `${64 * scale}px ${64 * scale}px`, // Жестко задаем и ширину, и высоту
        backgroundPosition: `-${8 * scale}px -${8 * scale}px`,
        imageRendering: 'pixelated'
      }} />
      
      {/* Слой шляпы/волос */}
      <div style={{
        position: 'absolute', top: 0, left: 0, width: '100%', height: '100%',
        backgroundImage: `url(${skinData})`,
        backgroundSize: `${64 * scale}px ${64 * scale}px`, 
        backgroundPosition: `-${40 * scale}px -${8 * scale}px`,
        imageRendering: 'pixelated'
      }} />
    </div>
  );
}