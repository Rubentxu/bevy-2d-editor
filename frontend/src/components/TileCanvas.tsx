import React, { useRef, useEffect, useState } from 'react';

interface TileCanvasProps {
  layerId: string;
  assetRef: string;
  tilesetImage: string; // image URL
  tileWidth: number;
  tileHeight: number;
  columns: number;
  gridWidth: number;  // number of tiles horizontally
  gridHeight: number; // number of tiles vertically
  mode: 'paint' | 'erase';
  selectedTile: { tilesetId: string; localIndex: number } | null;
  onPaint: (x: number, y: number) => void;
}

export const TileCanvas: React.FC<TileCanvasProps> = ({
  layerId,
  tilesetImage,
  tileWidth,
  tileHeight,
  columns,
  gridWidth,
  gridHeight,
  mode,
  selectedTile,
  onPaint,
}) => {
  const canvasRef = useRef<HTMLCanvasElement>(null);
  const [image, setImage] = useState<HTMLImageElement | null>(null);

  useEffect(() => {
    const img = new Image();
    img.src = tilesetImage;
    img.onload = () => setImage(img);
  }, [tilesetImage]);

  useEffect(() => {
    if (!canvasRef.current || !image) return;
    const canvas = canvasRef.current;
    const ctx = canvas.getContext('2d');
    if (!ctx) return;

    canvas.width = gridWidth * tileWidth;
    canvas.height = gridHeight * tileHeight;
    ctx.clearRect(0, 0, canvas.width, canvas.height);

    // Draw a background grid
    ctx.strokeStyle = '#333';
    ctx.lineWidth = 1;
    for (let x = 0; x <= gridWidth; x++) {
      ctx.beginPath();
      ctx.moveTo(x * tileWidth, 0);
      ctx.lineTo(x * tileWidth, canvas.height);
      ctx.stroke();
    }
    for (let y = 0; y <= gridHeight; y++) {
      ctx.beginPath();
      ctx.moveTo(0, y * tileHeight);
      ctx.lineTo(canvas.width, y * tileHeight);
      ctx.stroke();
    }
  }, [image, gridWidth, gridHeight, tileWidth, tileHeight]);

  const handleClick = (e: React.MouseEvent<HTMLCanvasElement>) => {
    const rect = canvasRef.current!.getBoundingClientRect();
    const x = Math.floor((e.clientX - rect.left) / tileWidth);
    const y = Math.floor((e.clientY - rect.top) / tileHeight);
    if (x >= 0 && x < gridWidth && y >= 0 && y < gridHeight) {
      onPaint(x, y);
    }
  };

  return (
    <canvas
      ref={canvasRef}
      onClick={handleClick}
      style={{ cursor: mode === 'paint' ? 'crosshair' : 'not-allowed' }}
    />
  );
};
