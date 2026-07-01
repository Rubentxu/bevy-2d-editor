import React, { useState, useEffect } from 'react';
import { listTilesets, createTileset, deleteTileset, type TilesetMetadata } from '../services/tilesets';

interface TilesetPanelProps {
  onSelectTileset: (tileset: TilesetMetadata) => void;
  selectedTilesetId: string | null;
}

export const TilesetPanel: React.FC<TilesetPanelProps> = ({
  onSelectTileset,
  selectedTilesetId,
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

  useEffect(() => {
    listTilesets().then(setTilesets).catch(console.error);
  }, []);

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
    </div>
  );
};
