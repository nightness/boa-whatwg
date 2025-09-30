//! Tests for the parent module.

use super::*;
use crate::Context;

#[test]
#[allow(clippy::missing_panics_doc)]
fn into_js_module() {
    use boa_engine::interop::{ContextData, JsRest};
    use boa_engine::{
        Context, IntoJsFunctionCopied, JsValue, Module, Source, UnsafeIntoJsFunction, js_string,
    };
    use boa_gc::{Gc, GcRefCell};
    use std::cell::RefCell;
    use std::rc::Rc;

    type ResultType = Gc<GcRefCell<JsValue>>;

    let loader = Rc::new(MapModuleLoader::default());
    let mut context = Context::builder()
        .module_loader(loader.clone())
        .build()
        .unwrap();

    let foo_count = Rc::new(RefCell::new(0));
    let bar_count = Rc::new(RefCell::new(0));
    let dad_count = Rc::new(RefCell::new(0));

    context.insert_data(Gc::new(GcRefCell::new(JsValue::undefined())));

    let module = unsafe {
        vec![
            (
                js_string!("foo"),
                {
                    let counter = foo_count.clone();
                    move || {
                        *counter.borrow_mut() += 1;

                        *counter.borrow()
                    }
                }
                .into_js_function_unsafe(&mut context),
            ),
            (
                js_string!("bar"),
                UnsafeIntoJsFunction::into_js_function_unsafe(
                    {
                        let counter = bar_count.clone();
                        move |i: i32| {
                            *counter.borrow_mut() += i;
                        }
                    },
                    &mut context,
                ),
            ),
            (
                js_string!("dad"),
                UnsafeIntoJsFunction::into_js_function_unsafe(
                    {
                        let counter = dad_count.clone();
                        move |args: JsRest<'_>, context: &mut Context| {
                            *counter.borrow_mut() += args
                                .into_iter()
                                .map(|i| i.try_js_into::<i32>(context).unwrap())
                                .sum::<i32>();
                        }
                    },
                    &mut context,
                ),
            ),
            (
                js_string!("send"),
                (move |value: JsValue, ContextData(result): ContextData<ResultType>| {
                    *result.borrow_mut() = value;
                })
                .into_js_function_copied(&mut context),
            ),
        ]
    }
    .into_js_module(&mut context);

    loader.insert("test", module);

    let source = Source::from_bytes(
        r"
            import * as test from 'test';
            let result = test.foo();
            test.foo();
            for (let i = 1; i <= 5; i++) {
                test.bar(i);
            }
            for (let i = 1; i < 5; i++) {
                test.dad(1, 2, 3);
            }

            test.send(result);
        ",
    );
    let root_module = Module::parse(source, None, &mut context).unwrap();

    let promise_result = root_module.load_link_evaluate(&mut context);
    context.run_jobs().unwrap();

    // Checking if the final promise didn't return an error.
    assert!(
        promise_result.state().as_fulfilled().is_some(),
        "module didn't execute successfully! Promise: {:?}",
        promise_result.state()
    );

    let result = context.get_data::<ResultType>().unwrap().borrow().clone();

    assert_eq!(*foo_count.borrow(), 2);
    assert_eq!(*bar_count.borrow(), 15);
    assert_eq!(*dad_count.borrow(), 24);
    assert_eq!(result.try_js_into(&mut context), Ok(1u32));
}

#[test]
fn can_throw_exception() {
    use boa_engine::{
        Context, IntoJsFunctionCopied, JsError, JsResult, JsValue, Module, Source, js_string,
    };
    use std::rc::Rc;

    let loader = Rc::new(MapModuleLoader::default());
    let mut context = Context::builder()
        .module_loader(loader.clone())
        .build()
        .unwrap();

    let module = vec![(
        js_string!("doTheThrow"),
        IntoJsFunctionCopied::into_js_function_copied(
            |message: JsValue| -> JsResult<()> { Err(JsError::from_opaque(message)) },
            &mut context,
        ),
    )]
    .into_js_module(&mut context);

    loader.insert("test", module);

    let source = Source::from_bytes(
        r"
            import * as test from 'test';
            try {
                test.doTheThrow('javascript');
            } catch(e) {
                throw 'from ' + e;
            }
        ",
    );
    let root_module = Module::parse(source, None, &mut context).unwrap();

    let promise_result = root_module.load_link_evaluate(&mut context);
    context.run_jobs().unwrap();

    // Checking if the final promise didn't return an error.
    assert_eq!(
        promise_result.state().as_rejected(),
        Some(&js_string!("from javascript").into())
    );
}