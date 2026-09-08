//! Runtime coordination buses — pure (no Bevy, no WASM) types owned by
//! EditorSession.
//!
//! These types live in editor-model so editor-application (which holds
//! EditorSession) can own them without creating an
//! editor-application -> editor-bevy dependency edge.

/// Fixed-capacity linear binary bus for command / event dispatch.
#[derive(Debug, Clone)]
pub struct LinearBus {
    buffer: Box<[u8]>,
}

impl LinearBus {
    /// Construct a fresh bus with BUS_CAPACITY bytes pre-allocated.
    pub fn new() -> Self {
        let mut buffer = vec![0u8; BUS_CAPACITY].into_boxed_slice();
        Self::set_write_offset(&mut buffer, 8);
        Self { buffer }
    }

    /// Pointer (offset into WebAssembly.Memory) of the buffer start.
    pub fn ptr(&self) -> u32 {
        self.buffer.as_ptr() as u32
    }

    /// Total byte length of the backing buffer.
    pub fn len(&self) -> u32 {
        self.buffer.len() as u32
    }

    fn get_write_offset(buf: &[u8]) -> usize {
        u32::from_le_bytes(buf[0..4].try_into().unwrap()) as usize
    }

    fn set_write_offset(buf: &mut [u8], offset: usize) {
        buf[0..4].copy_from_slice(&(offset as u32).to_le_bytes());
    }

    /// Drain all slots from the buffer, resetting the write offset to 8.
    pub fn drain(&mut self) -> Vec<(u16, Vec<u8>)> {
        let end = Self::get_write_offset(&self.buffer);
        Self::set_write_offset(&mut self.buffer, 8);
        let mut result = Vec::new();
        let mut pos = 8;
        while pos + 4 <= end && pos + 4 <= self.buffer.len() {
            let cmd_type = u16::from_le_bytes(self.buffer[pos..pos + 2].try_into().unwrap());
            let payload_len =
                u16::from_le_bytes(self.buffer[pos + 2..pos + 4].try_into().unwrap()) as usize;
            if pos + 4 + payload_len > self.buffer.len() {
                break;
            }
            let payload = self.buffer[pos + 4..pos + 4 + payload_len].to_vec();
            result.push((cmd_type, payload));
            pos += 4 + payload_len;
        }
        result
    }

    /// Reset the bus to the empty state (write offset = 8).
    pub fn reset(&mut self) {
        Self::set_write_offset(&mut self.buffer, 8);
    }

    /// Write one slot into the bus. Returns false if the bus is full.
    pub fn write(&mut self, event_type: u16, payload: &[u8]) -> bool {
        let write_offset = Self::get_write_offset(&self.buffer);
        let slot_size = 4 + payload.len();
        if write_offset + slot_size > self.buffer.len() {
            return false;
        }
        self.buffer[write_offset..write_offset + 2].copy_from_slice(&event_type.to_le_bytes());
        self.buffer[write_offset + 2..write_offset + 4]
            .copy_from_slice(&(payload.len() as u16).to_le_bytes());
        self.buffer[write_offset + 4..write_offset + 4 + payload.len()].copy_from_slice(payload);
        Self::set_write_offset(&mut self.buffer, write_offset + slot_size);
        true
    }
}

impl Default for LinearBus {
    fn default() -> Self {
        Self::new()
    }
}

const BUS_CAPACITY: usize = 65536;
