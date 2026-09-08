//!
//! Runtime coordination types — pure (no Bevy, no WASM) types owned by
//! EditorSession.
//!
//! These types live in editor-model so editor-application (which holds
//! EditorSession) can own them without creating an
//! editor-application → editor-bevy dependency edge (ADR-0030).

pub mod actuator_bus;
pub mod hot_reload;
pub mod linear_bus;
pub mod port_value;

pub use actuator_bus::{ActuatorBus, ActuatorOutput};
pub use hot_reload::{HotReloadRequest, PlayModeRequest};
pub use linear_bus::LinearBus;
pub use port_value::PortValue;
