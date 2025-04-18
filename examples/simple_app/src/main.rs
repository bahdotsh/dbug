use dbug::prelude::*;

#[dbug]
fn factorial(n: u64) -> u64 {
    if n <= 1 {
        break_here!();  // Breakpoint at base case
        return 1;
    } else {
        let result = n * factorial(n - 1);
        // We can't use attribute macros on statements, so use break_here instead
        break_here!();  // Breakpoint before returning
        return result;
    }
}

#[dbug]
fn fibonacci(n: u64) -> u64 {
    if n <= 1 {
        return n;
    } else {
        break_here!();  // Breakpoint in recursive case
        let result = fibonacci(n - 1) + fibonacci(n - 2);
        return result;
    }
}

fn main() {
    println!("Simple App - Testing dbug debugger");
    
    let fac5 = factorial(5);
    println!("Factorial of 5: {}", fac5);
    
    let fib10 = fibonacci(10);
    println!("Fibonacci of 10: {}", fib10);
} 