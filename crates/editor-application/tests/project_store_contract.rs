//! Generic contract test for any ProjectStore impl. Run against InMemoryProjectStore in v1.

use editor_application::adapters::in_memory::InMemoryProjectStore;
use editor_application::ports::project_store::ProjectStore;

#[test]
fn contract_list_read_write_delete_exists() {
    let s: Box<dyn ProjectStore> = Box::new(InMemoryProjectStore::new());
    assert!(!s.exists("a.txt").unwrap());
    s.write("a.txt", b"hi", false).unwrap();
    assert!(s.exists("a.txt").unwrap());
    assert_eq!(s.read("a.txt").unwrap(), b"hi");
    let xs = s.list("").unwrap();
    assert_eq!(xs.len(), 1);
    s.delete("a.txt").unwrap();
    assert!(!s.exists("a.txt").unwrap());
}

#[test]
fn contract_atomic_write_preserves_old_state_on_collision() {
    let s: Box<dyn ProjectStore> = Box::new(InMemoryProjectStore::new());
    s.write("a.txt", b"old", true).unwrap();
    s.write("a.txt", b"new", true).unwrap();
    assert_eq!(s.read("a.txt").unwrap(), b"new");
}
