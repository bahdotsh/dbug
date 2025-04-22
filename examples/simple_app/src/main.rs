use dbug::prelude::*;

// A macro to create a debug guard in a macro-friendly way
macro_rules! with_debug_guard {
    ($func_name:expr, $body:block) => {{
        struct _DbugGuard<'a> {
            fn_name: &'a str,
        }

        impl<'a> Drop for _DbugGuard<'a> {
            fn drop(&mut self) {
                dbug::_internal::exit_function(self.fn_name);
            }
        }

        let _guard = _DbugGuard {
            fn_name: $func_name,
        };
        dbug::_internal::enter_function($func_name);

        $body
    }};
}

// First try with the dbug macro, but with a fallback to manual instrumentation
// just in case the macro fails at compile time
#[cfg(not(feature = "use_manual_instrumentation"))]
#[dbug]
fn factorial(n: u64) -> u64 {
    if n <= 1 {
        // Use a macro for breakpoints
        break_here!();
        println!("Base case: n = {}", n);
        return 1;
    } else {
        let result = n * factorial(n - 1);
        break_here!();
        println!("Recursive case: n = {}, result = {}", n, result);
        return result;
    }
}

// Fallback manual implementation
#[cfg(feature = "use_manual_instrumentation")]
fn factorial(n: u64) -> u64 {
    with_debug_guard!("factorial", {
        if n <= 1 {
            dbug::_internal::break_point(file!(), line!(), 0);
            println!("Base case: n = {}", n);
            return 1;
        } else {
            let result = n * factorial(n - 1);
            dbug::_internal::break_point(file!(), line!(), 0);
            println!("Recursive case: n = {}, result = {}", n, result);
            return result;
        }
    })
}

// First try with the dbug macro, but with a fallback to manual instrumentation
#[cfg(not(feature = "use_manual_instrumentation"))]
#[dbug]
fn fibonacci(n: u64) -> u64 {
    if n <= 1 {
        println!("Base case: n = {}", n);
        return n;
    } else {
        break_here!();
        println!("Calculating fibonacci({})...", n);
        let result = fibonacci(n - 1) + fibonacci(n - 2);
        println!("fibonacci({}) = {}", n, result);
        return result;
    }
}

// Fallback manual implementation
#[cfg(feature = "use_manual_instrumentation")]
fn fibonacci(n: u64) -> u64 {
    with_debug_guard!("fibonacci", {
        if n <= 1 {
            println!("Base case: n = {}", n);
            return n;
        } else {
            dbug::_internal::break_point(file!(), line!(), 0);
            println!("Calculating fibonacci({})...", n);
            let result = fibonacci(n - 1) + fibonacci(n - 2);
            println!("fibonacci({}) = {}", n, result);
            return result;
        }
    })
}

fn main() {
    println!("Simple App - Testing dbug debugger");

    // For testing, use smaller numbers to reduce output
    let fac5 = factorial(5);
    println!("Factorial of 5: {}", fac5);

    let fib7 = fibonacci(7);
    println!("Fibonacci of 7: {}", fib7);
}
