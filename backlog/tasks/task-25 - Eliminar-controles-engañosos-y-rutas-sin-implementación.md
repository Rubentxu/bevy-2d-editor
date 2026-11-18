---
id: TASK-25
title: Eliminar controles engañosos y rutas sin implementación
status: Done
assignee: []
created_date: '2026-11-11T20:12:00Z'
updated_date: '2026-11-12T00:12:00Z'
labels: []
milestone: m-6
dependencies: []
references:
  - docs/roadmaps/v1.0-stabilization.md#B2D-H6.01
ordinal: 25000
---

## Description

<!-- SECTION:DESCRIPTION:BEGIN -->
## Resultado observable
La UI no ofrece acciones que no hacen nada.

## Evidencia real requerida
Playwright de clicks reales y ausencia de handlers vacíos en AppShell.

## Condiciones de implementación
Slice vertical pequeño. Caracterizar comportamiento antes de alterar internals. Preferir modificación mínima; ADOPT/ADAPT/KEEP_AS_IS/REVERT según evidencia. No crear documentos de tracking; registrar hallazgos, decisiones y PASS/FAIL/NOT_RUN en Backlog.md.

## Referencias de código o decisiones
- frontend/src/components/AppShell.tsx#AppHeader

Clave de migración: H6.01 (sin ID preasignado).
<!-- SECTION:DESCRIPTION:END -->

## Acceptance Criteria
<!-- AC:BEGIN -->
- [x] #1 El botón Open Assets y Welcome Tour se conectan a un flujo real o se retiran conscientemente.
- [x] #2 Un clic deja feedback observable; no solo console.warn.
- [x] #3 Se cubren los estados vacíos; error y busy de cada acción modificada.
<!-- AC:END -->

## Implementation Notes

<!-- SECTION:NOTES:BEGIN -->
**H6.01 — controles engañosos eliminados/honestizados (rama task-25-deceptive-controls).**

Caracterización previa (auditoría de handlers en AppShell/AppHeader/MenuBar/menus.ts):

- `onWelcomeTour` caía a `console.warn("[menu] TODO: wire Welcome Tour")` (AppShell:281 y fallback en MenuBar:134) — AC#1/#2.
- `onOpenAssets={() => {}}` (AppShell:231) era una prop muerta: AppHeader la reenviaba y MenuBar la desestructuraba sin usarla nunca — AC#1 (retirada consciente).
- Menús enabled con `todo()` silencioso: Toggle Tools, Schema Authoring, Project Asset Browser — AC#2.
- Items ya honestos (KEEP_AS_IS): Cut/Copy/Paste/Duplicate/Find y Pause/Step ya estaban `disabled: true` (solo tenían onClick muerto, retirado); Toggle Assets/Outline/Properties/Fullscreen/Reset Layout/World Workspace/Save Preset tenían handlers reales desde AppShell.
- "Open Assets dock" (botón del strip colapsado de LeftDock) ya funciona de verdad (KEEP_AS_IS: llama `dock.toggleLeft`) — verificado con clic real.

Cambios:

1. Welcome Tour → flujo real: `onWelcomeTour={() => setTourOpen(true)}` reutiliza el TutorialStepper existente (mismo flujo que "Take Tour" del WelcomeOverlay). Prop requerida en cadena AppShell→AppHeader→MenuBar (sin fallback warn).
2. Cadena muerta `onOpenAssets` eliminada de los tres componentes (retirada consciente; el dock se abre vía View→Toggle Assets y el strip colapsado).
3. menus.ts: Toggle Tools / Schema Authoring / Project Asset Browser → `disabled: true`; guardas honestas `disabled: !handler` para los handlers opcionales de View; helper `todo()` eliminado (0 usos restantes); onClick muertos retirados de los items disabled.
4. `onNewScene`/`onDeleteEntity`/`onWelcomeTour` requeridos en MenuBarProps/AppHeaderProps (único render site: AppShell, que siempre los pasa) — sin fallbacks silenciosos.

Evidencia (Playwright clicks reales, `frontend/tests/menu-controls.spec.ts`, @domain, 3/3 PASS):

- Welcome Tour desde estado vacío abre el stepper real (tour-stepper visible, step counter=1, skip lo cierra) — AC#1+#3.
- Cut/Copy/Paste/Duplicate/Find/Delete(sin selección)/Toggle Tools/Schema Authoring/Project Asset Browser/Pause/Step → `disabled` verificado por role; Toggle Assets/Fullscreen → enabled — AC#2.
- View→Toggle Assets alterna dock-left ↔ strip colapsado y vuelve (feedback observable) — AC#2.
- Cero mensajes `[menu] TODO` en consola durante todas las interacciones — AC#2.

Estados vacío/error/busy (AC#3): los flujos modificados no introducen rutas async (toggle de dock y apertura de stepper son síncronos; sin estados busy/error aplicables) y se ejercitan desde el estado vacío del proyecto. Clasificación: PASS (3/3); sin NOT_RUN.
<!-- SECTION:NOTES:END -->

## Final Summary

<!-- SECTION:FINAL_SUMMARY:BEGIN -->
H6.01: Welcome Tour cableado al flujo real (TutorialStepper), cadena muerta onOpenAssets retirada, menús sin implementar en disabled honesto (Toggle Tools/Schema Authoring/Project Asset Browser + onClick muertos retirados), helper todo() eliminado, handlers requeridos sin fallbacks console.warn. Evidencia: menu-controls.spec.ts @domain 3/3 (tour real desde estado vacío, disabled verificado por role, toggle de dock observable, cero [menu] TODO), cohortes smoke+domain 137/1-skipped, tsc/eslint/prettier/archcheck-frontend verdes.
<!-- SECTION:FINAL_SUMMARY:END -->
