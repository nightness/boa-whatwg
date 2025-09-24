use crate::{
    JsNativeErrorKind, JsValue, TestAction,
    error::JsNativeError,
    js_string,
    native_function::NativeFunction,
    object::{FunctionObjectBuilder, JsObject},
    property::{Attribute, PropertyDescriptor},
    run_test_actions,
};
use boa_macros::js_str;
use indoc::indoc;

#[allow(clippy::float_cmp)]
#[test]
fn arguments_object() {
    run_test_actions([
        TestAction::run(indoc! {r#"
                function jason(a, b) {
                    return arguments[0];
                }
            "#}),
        TestAction::assert_eq("jason(100, 6)", 100),
    ]);
}

#[test]
fn self_mutating_function_when_calling() {
    run_test_actions([
        TestAction::run(indoc! {r#"
                function x() {
                    x.y = 3;
                }
                x();
            "#}),
        TestAction::assert_eq("x.y", 3),
    ]);
}

#[test]
fn self_mutating_function_when_constructing() {
    run_test_actions([
        TestAction::run(indoc! {r#"
                function x() {
                    x.y = 3;
                }
                new x();
            "#}),
        TestAction::assert_eq("x.y", 3),
    ]);
}

#[test]
fn function_prototype() {
    run_test_actions([
        TestAction::assert_eq("Function.prototype.name", js_string!()),
        TestAction::assert_eq("Function.prototype.length", 0),
        TestAction::assert_eq("Function.prototype()", JsValue::undefined()),
        TestAction::assert_eq(
            "Function.prototype(1, '', new String(''))",
            JsValue::undefined(),
        ),
        TestAction::assert_native_error(
            "new Function.prototype()",
            JsNativeErrorKind::Type,
            "not a constructor",
        ),
    ]);
}

#[test]
fn function_prototype_call() {
    run_test_actions([TestAction::assert_eq(
        "Object.prototype.toString.call(new Error())",
        js_str!("[object Error]"),
    )]);
}

#[test]
fn function_prototype_call_throw() {
    run_test_actions([TestAction::assert_native_error(
        indoc! {r#"
            let call = Function.prototype.call;
            call(call)
        "#},
        JsNativeErrorKind::Type,
        "undefined is not a function",
    )]);
}

#[test]
fn function_prototype_call_multiple_args() {
    run_test_actions([
        TestAction::run(indoc! {r#"
            function f(a, b) {
                this.a = a;
                this.b = b;
            }
            let o = {a: 0, b: 0};
            f.call(o, 1, 2);
        "#}),
        TestAction::assert_eq("o.a", 1),
        TestAction::assert_eq("o.b", 2),
    ]);
}

#[test]
fn function_prototype_apply() {
    run_test_actions([
        TestAction::run("const numbers = [6, 7, 3, 4, 2]"),
        TestAction::assert_eq("Math.max.apply(null, numbers)", 7),
        TestAction::assert_eq("Math.min.apply(null, numbers)", 2),
    ]);
}

#[test]
fn function_prototype_apply_on_object() {
    run_test_actions([
        TestAction::run(indoc! {r#"
                function f(a, b) {
                    this.a = a;
                    this.b = b;
                }
                let o = {a: 0, b: 0};
                f.apply(o, [1, 2]);
            "#}),
        TestAction::assert_eq("o.a", 1),
        TestAction::assert_eq("o.b", 2),
    ]);
}

#[test]
fn closure_capture_clone() {
    run_test_actions([
    TestAction::inspect_context(|ctx| {
            let string = js_string!("Hello");
            let object = JsObject::with_object_proto(ctx.intrinsics());
            object
                .define_property_or_throw(
                    js_string!("key"),
                    PropertyDescriptor::builder()
                        .value(js_string!(" world!"))
                        .writable(false)
                        .enumerable(false)
                        .configurable(false),
                    ctx,
                )
                .unwrap();

            let func = FunctionObjectBuilder::new(
                ctx.realm(),
                NativeFunction::from_copy_closure_with_captures(
                    |_, _, captures, context| {
                        let (string, object) = &captures;

                        let hw = js_string!(
                            string,
                            &object
                                .__get_own_property__(
                                    &js_string!("key").into(),
                                    &mut context.into()
                                )?
                                .and_then(|prop| prop.value().and_then(JsValue::as_string))
                                .ok_or_else(
                                    || JsNativeError::typ().with_message("invalid `key` property")
                                )?
                        );
                        Ok(hw.into())
                    },
                    (string, object),
                ),
            )
            .name(js_str!("closure"))
            .build();

            ctx.register_global_property(js_str!("closure"), func, Attribute::default())
                .unwrap();
        }),
        TestAction::assert_eq("closure()", js_str!("Hello world!")),
    ]);
}

#[test]
fn function_constructor_early_errors_super() {
    run_test_actions([
        TestAction::assert_native_error(
            "Function('super()')()",
            JsNativeErrorKind::Syntax,
            "invalid `super` call",
        ),
        TestAction::assert_native_error(
            "Function('super.a')()",
            JsNativeErrorKind::Syntax,
            "invalid `super` reference",
        ),
    ]);
}

// Temporary debug test to inspect the types and values passed to create_dynamic_function.
// This writes a debug file `engines/boa/debug-create-dynamic.txt` in the workspace so
// we can inspect the exact JsValue representations observed at runtime.
#[test]
fn debug_create_dynamic_function_inspect_args() {
    use crate::run_test_actions;
    use crate::TestAction;

    run_test_actions([
        TestAction::inspect_context(|ctx| {
            use crate::builtins::function::BuiltInFunctionObject;
            use crate::js_string;
            use std::fs::OpenOptions;
            use std::io::Write;

            // Construct arguments equivalent to Function('super()')
            let body = crate::js_string!("super()");
            let args = [crate::JsValue::new(body.clone())];

            // Use the intrinsic function constructor as the `constructor` argument.
            let constructor = ctx.intrinsics().constructors().function().constructor();

            // Call create_dynamic_function directly and capture the result.
            let res = BuiltInFunctionObject::create_dynamic_function(
                constructor,
                &crate::JsValue::undefined(),
                &args,
                false,
                false,
                ctx,
            );

            // Write debugging info to a file in the crate root of engines/boa so it's easy to find.
            if let Ok(mut f) = OpenOptions::new()
                .create(true)
                .write(true)
                .truncate(true)
                .open("debug-create-dynamic.txt")
            {
                let _ = writeln!(f, "create_dynamic_function result: {:?}\n", res);
                let _ = writeln!(f, "args[0] (JsValue debug): {:?}\n", args[0]);
                if let Ok(s) = args[0].to_string(ctx) {
                    let _ = writeln!(f, "args[0] as string: {:?}\n", s.to_std_string_escaped());
                }
            }
            // inspect_context expects unit return
            ()
        }),
    ]);
}

// Debug: evaluate the JS string via the engine's normal eval path and write result/error
#[test]
fn debug_eval_function_constructor_via_eval() {
    use crate::run_test_actions;
    use crate::TestAction;

    run_test_actions([
        TestAction::inspect_context(|ctx| {
            use std::fs::OpenOptions;
            use std::io::Write;

            // Evaluate the JS expression using the engine's normal evaluation path.
            let code = "Function('super()')()";
            use boa_parser::Source;
            let src = Source::from_bytes(code);
            let res = ctx.eval(src);

            if let Ok(mut f) = OpenOptions::new()
                .create(true)
                .write(true)
                .truncate(true)
                .open("debug-eval-function-constructor.txt")
            {
                let _ = writeln!(f, "eval code = {}", code);
                let _ = writeln!(f, "result = {:?}", res);
                // Also capture what `Function` is in the global environment
                let ty_src = "typeof Function";
                let ty = ctx.eval(Source::from_bytes(ty_src));
                let _ = writeln!(f, "typeof Function -> {:?}", ty);

                let to_str_src = "Function.toString()";
                let to_str = ctx.eval(Source::from_bytes(to_str_src));
                let _ = writeln!(f, "Function.toString() -> {:?}", to_str);

                let proto_src = "Object.prototype.toString.call(Function)";
                let proto = ctx.eval(Source::from_bytes(proto_src));
                let _ = writeln!(f, "Object.prototype.toString.call(Function) -> {:?}", proto);
            }

            ()
        }),
    ]);
}
