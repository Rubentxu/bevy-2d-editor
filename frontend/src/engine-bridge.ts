const CMD_MOVE_SPRITE = 1;
const EVT_SPRITE_POSITION = 1;
const EVT_FPS = 2;

let wasm: any = null;
let wasmMemory: WebAssembly.Memory | null = null;
let cmdView: DataView | null = null;
let evtView: DataView | null = null;
let lastMemSize = 0;
let frameCallback: ((type: number, payload: DataView) => void) | null = null;
let engineReady = false;

function refreshViews() {
  if (!wasm || !wasmMemory) return;
  const currentSize = wasmMemory.buffer.byteLength;
  if (currentSize !== lastMemSize) {
    lastMemSize = currentSize;
    cmdView = new DataView(
      wasmMemory.buffer,
      wasm.get_command_bus_ptr(),
      wasm.get_command_bus_len()
    );
    evtView = new DataView(
      wasmMemory.buffer,
      wasm.get_event_bus_ptr(),
      wasm.get_event_bus_len()
    );
  }
}

function pollEvents() {
  if (!evtView || !frameCallback) return;
  const writeOffset = evtView.getUint32(0, true);
  let pos = 8;
  while (pos + 4 <= writeOffset) {
    const type = evtView.getUint16(pos, true);
    const len = evtView.getUint16(pos + 2, true);
    if (pos + 4 + len > evtView.byteLength) break;
    const payload = new DataView(
      evtView.buffer,
      evtView.byteOffset + pos + 4,
      len
    );
    frameCallback(type, payload);
    pos += 4 + len;
  }
  evtView.setUint32(0, 8, true);
}

export async function initEngine(
  canvasId: string,
  onEvent: (type: number, payload: DataView) => void
): Promise<void> {
  console.log("[bridge] Loading WASM module...");

  const wasmModule = await import("./wasm/editor_core.js");
  await wasmModule.default();
  wasm = wasmModule;
  wasmMemory = (wasmModule as any).__wasm.memory;
  console.log("[bridge] WASM module loaded, memory size:", wasmMemory.buffer.byteLength);

  frameCallback = onEvent;

  (window as any).onFrameEnd = () => {
    refreshViews();
    pollEvents();
  };

  // Expose load_scene_json for testing
  (window as any).load_scene_json = (json: string) => wasm.load_scene_json(json);
  // Expose dispatch_command for testing (typed command system)
  (window as any).dispatch_command = (json: string) => wasm.dispatch_command(json);
  // Expose undo/redo for testing (operation log)
  (window as any).undo = () => wasm.undo();
  (window as any).redo = () => wasm.redo();
  (window as any).get_log_state = () => wasm.get_log_state();
  // Expose OPFS persistence for testing
  (window as any).save_scene = (name: string) => wasm.save_scene(name);
  (window as any).load_scene = (name: string) => wasm.load_scene(name);
  (window as any).list_scenes = () => wasm.list_scenes();
  (window as any).project_exists = () => wasm.project_exists();
  // Expose schema registry persistence for testing
  (window as any).save_schema = (typeId: string) => wasm.save_schema(typeId);
  (window as any).load_schema = (typeId: string) => wasm.load_schema(typeId);
  (window as any).delete_schema = (typeId: string) => wasm.delete_schema(typeId);
  (window as any).list_schemas = () => wasm.list_schemas();
  (window as any).register_schema = (json: string) => wasm.register_schema_from_json(json);
  (window as any).unregister_schema = (typeId: string) => wasm.unregister_schema(typeId);
  (window as any).is_builtin_type = (typeId: string) => wasm.is_builtin_type(typeId);
  (window as any).combined_registry_size = () => wasm.combined_registry_size();
  (window as any).load_project = () => wasm.load_project();
  // Expose OPFS bridge functions for wasm_bindgen externs
  const opfs = await import("./opfs-bridge");
  (window as any).opfs_save_file = opfs.opfsSaveFile;
  (window as any).opfs_load_file = opfs.opfsLoadFile;
  (window as any).opfs_list_files = opfs.opfsListFiles;
  (window as any).opfs_exists = opfs.opfsExists;
  (window as any).opfs_delete_file = opfs.opfsDeleteFile;

  // Step 1: Create buses BEFORE starting engine
  wasm.create_buses();
  console.log("[bridge] Buses created");

  // Step 2: Set up DataView references to shared memory
  refreshViews();
  console.log(
    `[bridge] cmdView: ptr=${wasm.get_command_bus_ptr()} len=${wasm.get_command_bus_len()}`
  );
  console.log(
    `[bridge] evtView: ptr=${wasm.get_event_bus_ptr()} len=${wasm.get_event_bus_len()}`
  );

  // Step 3: Start Bevy engine (deferred so this promise can resolve)
  engineReady = true;
  setTimeout(() => {
    try {
      console.log("[bridge] Starting Bevy engine...");
      wasm.start_engine(canvasId);
      console.log("[bridge] start_engine returned normally");
    } catch (e) {
      console.error("[bridge] start_engine threw:", e);
    }
  }, 0);

  console.log("[bridge] initEngine resolved");
}

export function sendMoveSprite(x: number, y: number) {
  if (!cmdView) {
    console.warn("[bridge] cmdView not ready");
    return;
  }
  refreshViews();
  const writeOffset = cmdView.getUint32(0, true);
  if (writeOffset + 12 > cmdView.byteLength) {
    console.warn("[bridge] Command bus full");
    return;
  }
  cmdView.setUint16(writeOffset, CMD_MOVE_SPRITE, true);
  cmdView.setUint16(writeOffset + 2, 8, true);
  cmdView.setFloat32(writeOffset + 4, x, true);
  cmdView.setFloat32(writeOffset + 8, y, true);
  cmdView.setUint32(0, writeOffset + 12, true);
}

/**
 * Dispatch a typed Command to the editor core.
 * @param envelope The CommandEnvelope (command + metadata) as JSON string.
 * @returns JSON string with CommandResult (inverse + snapshot).
 */
export async function dispatchCommand(envelope: object): Promise<string> {
  const json = JSON.stringify(envelope);
  return (window as any).dispatch_command(json);
}

/**
 * Undo the last command. Returns the new document snapshot as JSON string.
 */
export async function undo(): Promise<string> {
  return (window as any).undo();
}

/**
 * Redo the next command. Returns the new document snapshot as JSON string.
 */
export async function redo(): Promise<string> {
  return (window as any).redo();
}

/**
 * Get operation log metadata (size, can_undo, can_redo, cursor).
 */
export async function getLogState(): Promise<{ size: number; can_undo: boolean; can_redo: boolean; cursor: number }> {
  const json = (window as any).get_log_state();
  return JSON.parse(json);
}

export function isEngineReady() {
  return engineReady;
}

export { EVT_SPRITE_POSITION, EVT_FPS };
