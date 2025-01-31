use godot::prelude::*;

#[derive(GodotClass)]
#[class(base=RefCounted,init)]
struct StagSanityObject {
    #[export]
    value: i32,
}

#[godot_api]
impl StagSanityObject {
    #[func]
    fn stringify_int(&self, int: i32) -> GString {
        int.to_variant().stringify()
    }

    #[func] // static
    fn return_int(a: i32) -> i32 {
        a
    }

    #[func]
    fn return_int_via_callable_call(a: i32) -> Variant {
        let callable = Callable::from_local_static("StagSanityObject", "return_int");

        callable.call(&[Variant::from(a)])
    }

    #[func]
    fn return_int_via_callable_bind(a: i32) -> Variant {
        let callable = Callable::from_local_static("StagSanityObject", "return_int");
        let callable = callable.bind(&[Variant::from(a)]);

        callable.call(&[])
    }
}
