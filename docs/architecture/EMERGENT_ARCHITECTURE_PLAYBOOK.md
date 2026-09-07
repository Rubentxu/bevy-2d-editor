# Emergent Architecture Playbook

## Objetivo

Permitir que la arquitectura evolucione con el conocimiento adquirido durante el desarrollo sin convertir “arquitectura emergente” en ausencia de reglas.

## Modelo de decisión

Cada hito estructural sigue cinco pasos:

1. **Observation**: evidencia del problema actual.
2. **Hypothesis**: cambio mínimo que debería mejorar una quality attribute.
3. **Experiment**: spike o slice vertical con alcance limitado.
4. **Evidence**: tests, métricas, bundle/runtime impact, ergonomía y complejidad resultante.
5. **Checkpoint**: `adopt`, `adapt`, `defer` o `revert`.

## Clasificación de decisiones

### Locked

Cambiar solo mediante nuevo ADR y evidencia fuerte:

- authoring model como fuente de verdad;
- identidad persistente independiente de Bevy Entity;
- cambios semánticos reversibles;
- ChangeSet para operaciones transversales/revisables;
- separación de domain/application de infraestructura.

### Directional

Dirección deseada, implementación abierta:

- una composición WASM;
- backend TypeScript tipado;
- frontend por feature slices;
- documentos activos como unidad de workspace;
- GraphKernel reutilizable.

### Experimental

Debe validarse por spike:

- GATs en GraphKernel;
- virtualizador concreto de Hierarchy;
- generación automática de bindings TS/Rust;
- filesystem/native companion;
- event model interno del frontend;
- forma exacta del BSN AST/IR lossless.

## Architecture runway

Solo se construye infraestructura cuando existe un consumidor real dentro de los siguientes 1–2 hitos.

No se acepta:

- crear un framework genérico “por si acaso”;
- crear traits sin dos consumidores plausibles o una frontera clara;
- crear eventos para sustituir llamadas simples sin beneficio observable;
- mover código solo para alcanzar una estructura de carpetas ideal.

## Regla del dolor

Un hito se detiene y se reevalúa cuando ocurre uno de estos casos:

- para extraer una responsabilidad hay que cambiar tres o más formatos persistidos;
- aparecen más adapters temporales que componentes eliminados;
- una abstraction obliga a `downcast`, strings mágicos o `Any` para usos normales;
- la latencia/allocations empeoran por encima del presupuesto sin compensación;
- los tests requieren más mocks ambientales que antes;
- el mismo concepto sigue existiendo en dos fuentes de verdad después del hito.

## Decision checkpoint template

```markdown
### Checkpoint Hx.y

Evidence:
- tests:
- perf:
- dependency graph:
- UX:
- operational complexity:

Decision: ADOPT | ADAPT | DEFER | REVERT

Reason:

Next experiment:
```

## Fitness over diagrams

Los diagramas describen intención; CI demuestra realidad.

Cualquier regla crítica de dependencias debe poder verificarse con:

- `cargo metadata`;
- inspección de imports permitidos;
- TypeScript boundaries;
- tests de contrato;
- tests de serialización;
- UAT observable.

## Arquitectura y producto

No se acepta una mejora arquitectónica que degrade de forma permanente un workflow existente. Si una mejora requiere una transición, ambas rutas pueden coexistir temporalmente con:

- una fecha/condición de retirada;
- parity tests;
- telemetría o evidencia de callers;
- rollback claro.

