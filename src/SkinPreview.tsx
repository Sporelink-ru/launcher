import { useEffect, useRef } from 'react';
import * as skinview3d from 'skinview3d';

export default function SkinPreview({ skinUrl, capeUrl }: { skinUrl: string | null, capeUrl: string | null }) {
  const canvasRef = useRef<HTMLCanvasElement>(null);
  const viewerRef = useRef<skinview3d.SkinViewer | null>(null);

  useEffect(() => {
    if (canvasRef.current && skinUrl) {
      if (viewerRef.current) viewerRef.current.dispose();
      viewerRef.current = new skinview3d.SkinViewer({ canvas: canvasRef.current, width: 200, height: 250, skin: skinUrl });
      viewerRef.current.controls.enableZoom = false;
      viewerRef.current.animation = new skinview3d.WalkingAnimation();
      
      // Загружаем плащ, если он есть!
      if (capeUrl) {
        viewerRef.current.loadCape(capeUrl).catch(console.error);
      }
    }
    return () => { viewerRef.current?.dispose(); };
  }, [skinUrl, capeUrl]);

  if (!skinUrl) return <div style={{ marginTop: 20, color: '#94a3b8', fontSize: 13 }}>Скин не загружен</div>;
  return (
    <div style={{ marginTop: 20, display: 'flex', justifyContent: 'center', minHeight: 250 }}>
      <canvas ref={canvasRef} style={{ width: 200, height: 250, display: 'block' }} />
    </div>
  );
}