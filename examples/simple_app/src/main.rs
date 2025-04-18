use dbug::prelude::*;

#[dbug]
fn factorial(n: u64) -> u64 {
    if n <= 1 {
        dbug::break_here!(); // Breakpoint at base case
        1
    } else {
        let result = n * factorial(n - 1);
        #[dbug::break_at]
        let final_result = result; // Breakpoint before returning
        final_result
    }
}

#[dbug]
fn fibonacci(n: u64) -> u64 {
    if n <= 1 {
        n
    } else {
        dbug::break_here!(); // Breakpoint in recursive case
        fibonacci(n - 1) + fibonacci(n - 2)
    }
}

fn main() {
    println!("Simple App - Testing dbug debugger");
    
    let fac5 = factorial(5);
    println!("Factorial of 5: {}", fac5);
    
    let fib10 = fibonacci(10);
    println!("Fibonacci of 10: {}", fib10);
} 