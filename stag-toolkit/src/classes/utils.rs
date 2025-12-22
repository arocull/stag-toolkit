use godot::prelude::*;

/// Sets the editor lock metadata tag on the given node, so it cannot be selected.
pub fn editor_lock(mut node: Gd<Node>, lock: bool) {
    node.set_meta("_edit_lock_", &Variant::from(lock));
}
