/**
 * Code-export service — wraps the `export_code` WASM binding.
 * Returns `{ source: string, warnings: ExportWarning[] }`.
 */

export interface ExportWarning {
  entity_stable_id: string | null;
  component_type_id: string | null;
  message: string;
}

export interface ExportResult {
  source: string;
  warnings: ExportWarning[];
}

export async function exportToRust(sceneJson: string): Promise<ExportResult> {
  const result = await (window as any).export_code(sceneJson);
  return result as ExportResult;
}
