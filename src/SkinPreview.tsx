import { useEffect, useRef, useState } from 'react';
import * as skinview3d from 'skinview3d';

export default function SkinPreview({ skinUrl }: { skinUrl: string | null }) {
  const canvasRef = useRef<HTMLCanvasElement>(null);
  const viewerRef = useRef<skinview3d.SkinViewer | null>(null);
  const [error, setError] = useState(false);

  useEffect(() => {
    if (canvasRef.current && skinUrl) {
      try {
        if (viewerRef.current) viewerRef.current.dispose();

        viewerRef.current = new skinview3d.SkinViewer({
          canvas: canvasRef.current,
          width: 200,
          height: 250,
          skin: skinUrl,
        });

        viewerRef.current.controls.enableZoom = false;
        viewerRef.current.animation = new skinview3d.WalkingAnimation();
      } catch (err) {
        console.error("Skin render error:", err);
        setError(true);
      }
    }

    return () => {
      viewerRef.current?.dispose();
    };
  }, [skinUrl]);

  if (!skinUrl) {
    return <div style={{ marginTop: 20, color: '#94a3b8', fontSize: 13 }}>Скин не загружен</div>;
  }

  if (error) {
    return <div style={{ marginTop: 20, color: '#ef4444', fontSize: 13 }}>Ошибка загрузки скина</div>;
  }

  return (
    <div style={{ marginTop: 20, display: 'flex', justifyContent: 'center', minHeight: 250 }}>
      <canvas ref={canvasRef} style={{ width: 200, height: 250, display: 'block' }} />
    </div>
  );
}