use godot::prelude::*;

pub fn editor_lock(mut node: Gd<Node>, lock: bool) {
    node.set_meta("_edit_lock_", &Variant::from(lock));
}
