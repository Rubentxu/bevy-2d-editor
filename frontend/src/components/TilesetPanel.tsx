import React, { useState, useEffect, useCallback } from 'react';
import { listTilesets, createTileset, deleteTileset, paintTile, eraseTile, type TilesetMetadata } from '../services/tilesets';
import { type SceneAssetDocument, type TileLayerPayload } from '../services/scene-assets';
import { TileCanvas } from './TileCanvas';

// Default canvas height in tile rows. The Rust TileLayer uses a sparse grid
// without fixed dimensions; this constant sets the visible paint surface
// height until a proper grid_width/grid_height schema field ships.
const DEFAULT_CANVAS_GRID_HEIGHT = 50;

interface TilesetPanelProps {
  onSelectTileset: (tileset: TilesetMetadata) => void;
  selectedTilesetId: string | null;
  assetDoc: SceneAssetDocument | null;
  activeAssetLogicalPath: string | null;
}

export const TilesetPanel: React.FC<TilesetPanelProps> = ({
  onSelectTileset,
  selectedTilesetId,
  assetDoc,
  activeAssetLogicalPath,
}) => {
  const [tilesets, setTilesets] = useState<TilesetMetadata[]>([]);
  const [showCreate, setShowCreate] = useState(false);
  const [newTileset, setNewTileset] = useState({
    name: '',
    imageRef: '',
    tileWidth: 16,
    tileHeight: 16,
    columns: 16,
    spacing: 0,
  });
  const [selectedTileLayerId, setSelectedTileLayerId] = useState<string | null>(null);
  const [paintMode, setPaintMode] = useState<'paint' | 'erase'>('paint');
  const [selectedTileIndex, setSelectedTileIndex] = useState<number | null>(null);

  // Derive TileLayers from assetDoc
  const tileLayers: TileLayerPayload[] =
    (assetDoc?.layers?.filter((l) => l.kind === 'tile') as TileLayerPayload[]) ?? [];

  // Selected TileLayer and Tileset
  const selectedTileLayer = tileLayers.find((l) => l.id === selectedTileLayerId) ?? null;
  const selectedTileset = tilesets.find((ts) => ts.id === selectedTilesetId) ?? null;

  useEffect(() => {
    listTilesets().then(setTilesets).catch(console.error);
  }, []);

  // Reset layer selection when asset changes
  useEffect(() => {
    setSelectedTileLayerId(null);
    setSelectedTileIndex(null);
  }, [assetDoc?.logical_path]);

  const handleCreate = async () => {
    try {
      await createTileset(
        newTileset.name,
        newTileset.imageRef,
        newTileset.tileWidth,
        newTileset.tileHeight,
        newTileset.columns,
        newTileset.spacing
      );
      const updated = await listTilesets();
      setTilesets(updated);
      setShowCreate(false);
    } catch (e) {
      console.error('Failed to create tileset:', e);
    }
  };

  const handlePaint = useCallback(
    async (x: number, y: number) => {
      if (!activeAssetLogicalPath || !selectedTileLayerId || !selectedTilesetId) return;
      try {
        if (paintMode === 'paint' && selectedTileIndex !== null) {
          await paintTile(activeAssetLogicalPath, selectedTileLayerId, x, y, selectedTilesetId, selectedTileIndex);
        } else if (paintMode === 'erase') {
          await eraseTile(activeAssetLogicalPath, selectedTileLayerId, x, y);
        }
      } catch (e) {
        console.error('Paint failed:', e);
      }
    },
    [activeAssetLogicalPath, selectedTileLayerId, selectedTilesetId, selectedTileIndex, paintMode]
  );

  // Show tile canvas when both a tile layer and tileset are selected
  const showTileCanvas = !!selectedTileLayer && !!selectedTileset;

  return (
    <div className="tileset-panel">
      <h3>Tilesets</h3>
      <button onClick={() => setShowCreate(!showCreate)}>+ New Tileset</button>

      {showCreate && (
        <div className="create-form">
          <input placeholder="Name" value={newTileset.name} onChange={e => setNewTileset({...newTileset, name: e.target.value})} />
          <input placeholder="Image path (e.g. assets/tilesets/grass.png)" value={newTileset.imageRef} onChange={e => setNewTileset({...newTileset, imageRef: e.target.value})} />
          <div className="grid-dims">
            <input type="number" placeholder="Tile W" value={newTileset.tileWidth} onChange={e => setNewTileset({...newTileset, tileWidth: +e.target.value})} />
            <input type="number" placeholder="Tile H" value={newTileset.tileHeight} onChange={e => setNewTileset({...newTileset, tileHeight: +e.target.value})} />
            <input type="number" placeholder="Columns" value={newTileset.columns} onChange={e => setNewTileset({...newTileset, columns: +e.target.value})} />
            <input type="number" placeholder="Spacing" value={newTileset.spacing} onChange={e => setNewTileset({...newTileset, spacing: +e.target.value})} />
          </div>
          <button onClick={handleCreate}>Create</button>
        </div>
      )}

      <ul>
        {tilesets.map(ts => (
          <li
            key={ts.id}
            className={ts.id === selectedTilesetId ? 'selected' : ''}
            onClick={() => onSelectTileset(ts)}
          >
            {ts.name}
            <button onClick={e => { e.stopPropagation(); deleteTileset(ts.id).then(() => setTilesets(tilesets.filter(t => t.id !== ts.id))); }}>×</button>
          </li>
        ))}
      </ul>

      {/* Tile Layer picker — only in asset-authoring with level assets */}
      {tileLayers.length > 0 && (
        <div className="tile-layer-picker">
          <h4>Tile Layer</h4>
          <select
            value={selectedTileLayerId ?? ''}
            onChange={(e) => setSelectedTileLayerId(e.target.value || null)}
          >
            <option value="">— select layer —</option>
            {tileLayers.map((layer) => (
              <option key={layer.id} value={layer.id}>
                {layer.name}
              </option>
            ))}
          </select>
        </div>
      )}

      {/* Tile Canvas — shown when layer + tileset are both selected */}
      {showTileCanvas && (
        <>
          <div className="tile-canvas-toolbar">
            <button
              className={paintMode === 'paint' ? 'active' : ''}
              onClick={() => setPaintMode('paint')}
            >
              Paint
            </button>
            <button
              className={paintMode === 'erase' ? 'active' : ''}
              onClick={() => setPaintMode('erase')}
            >
              Erase
            </button>
            <span style={{ fontSize: 11, color: '#666', marginLeft: 8 }}>
              Pick tile:
            </span>
            <input
              type="number"
              min={0}
              value={selectedTileIndex ?? ''}
              onChange={(e) => setSelectedTileIndex(e.target.value ? parseInt(e.target.value) : null)}
              placeholder="index"
              style={{ width: 50 }}
            />
          </div>
          <TileCanvas
            layerId={selectedTileLayer!.id}
            assetRef={activeAssetLogicalPath ?? ''}
            tilesetImage={selectedTileset!.image_ref}
            tileWidth={selectedTileset!.tile_width}
            tileHeight={selectedTileset!.tile_height}
            columns={selectedTileset!.columns}
            gridWidth={selectedTileset!.columns}
            // Default 50 rows of canvas height. The Rust TileLayer data model
            // uses a sparse grid (HashMap) without fixed dimensions; adding
            // grid_width/grid_height to the schema is a separate cycle. Until
            // then, this constant sets the visible canvas height for the paint
            // surface. Wires together with the erase_tile wiring (HD-N3 fix).
            gridHeight={DEFAULT_CANVAS_GRID_HEIGHT}
            mode={paintMode}
            selectedTile={
              selectedTileIndex !== null
                ? { tilesetId: selectedTilesetId!, localIndex: selectedTileIndex }
                : null
            }
            onPaint={handlePaint}
          />
        </>
      )}

      {selectedTileLayer && !selectedTileset && (
        <p style={{ fontSize: 12, color: '#666', margin: '8px 0' }}>
          Select a tileset to paint on &quot;{selectedTileLayer.name}&quot;
        </p>
      )}
    </div>
  );
};
