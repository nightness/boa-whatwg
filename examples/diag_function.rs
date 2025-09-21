use boa_engine::{Context, Source};
use boa_runtime::register;
use boa_runtime::extensions::ConsoleExtension;

fn main() {
    let mut context = Context::default();
    // register runtime extensions similar to test harness
    register(ConsoleExtension::default(), None, &mut context).expect("failed to register runtime");

    let src = "Function('super()')()";
    match context.eval(Source::from_bytes(src)) {
        Ok(v) => println!("Result: {}", v.display()),
        Err(e) => eprintln!("Error: {}", e),
    }
}
