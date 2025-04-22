use proc_macro::TokenStream;
use quote::{quote, ToTokens};
use syn::{parse_macro_input, parse_quote, Block, Expr, ItemFn, Stmt};

/// Marks a function for debugging with dbug
///
/// This macro processes a function and inserts instrumentation code
/// to enable debugging capabilities.
///
/// # Example
///
/// ```
/// use dbug::prelude::*;
///
/// #[dbug]
/// fn my_function(x: i32) -> i32 {
///     let y = x * 2;
///     dbug::break_here!();  // A debug point
///     y + 10
/// }
/// ```
#[proc_macro_attribute]
pub fn dbug(_attr: TokenStream, item: TokenStream) -> TokenStream {
    // Parse the input function
    let result = parse_macro_input!(item as ItemFn);

    // Get function details
    let mut input_fn = result;
    let fn_name = &input_fn.sig.ident;
    let fn_name_str = fn_name.to_string();

    // Add exit instrumentation using a guard pattern with Drop
    // to ensure it's called on all exit paths, including early returns and panics
    let block = &input_fn.block;

    let new_block: Block = parse_quote! {{
        // Create a guard struct to handle function exit
        struct _DbugGuard<'a> {
            fn_name: &'a str,
        }

        impl<'a> Drop for _DbugGuard<'a> {
            fn drop(&mut self) {
                ::dbug::_internal::exit_function(self.fn_name);
            }
        }

        // Create the guard - will be dropped when the function exits
        let _guard = _DbugGuard { fn_name: #fn_name_str };

        // Notify function entry
        ::dbug::_internal::enter_function(#fn_name_str);

        // Original function body continues here
        #block
    }};

    // Replace the function block with our instrumented block
    input_fn.block = Box::new(new_block);

    // Convert back to TokenStream
    let output = input_fn.to_token_stream();
    output.into()
}

/// A macro for marking a breakpoint in code
///
/// # Example
///
/// ```
/// use dbug::prelude::*;
///
/// fn my_function() {
///     let x = 42;
///     dbug::break_here!();  // This will pause execution when running with dbug
///     println!("After breakpoint");
/// }
/// ```
#[proc_macro]
pub fn break_here(_input: TokenStream) -> TokenStream {
    let output = quote! {
        {
            ::dbug::_internal::break_point(file!(), line!(), column!());
        }
    };

    output.into()
}

/// Attribute to mark a line of code as a debug point
///
/// # Example
///
/// ```
/// fn my_function() {
///     let x = 42;
///     #[dbug::break_at]
///     let y = x + 1;  // This line will have a breakpoint
/// }
/// ```
#[proc_macro_attribute]
pub fn break_at(_attr: TokenStream, item: TokenStream) -> TokenStream {
    let result = syn::parse::<Stmt>(item.clone());

    match result {
        Ok(item_ast) => {
            let output = quote! {
                {
                    ::dbug::_internal::break_point(file!(), line!(), column!());
                    #item_ast
                }
            };

            output.into()
        }
        Err(err) => {
            // Return the original unmodified stream if we fail to parse
            eprintln!("Error in dbug::break_at macro: {}", err);
            item
        }
    }
}

/// Register a variable with the debugger
///
/// # Example
///
/// ```
/// use dbug::prelude::*;
///
/// fn my_function() {
///     let x = 42;
///     dbug::register_var!(x);  // This will register x with the debugger
/// }
/// ```
#[proc_macro]
pub fn register_var(input: TokenStream) -> TokenStream {
    let result = syn::parse::<syn::Ident>(input.clone());

    match result {
        Ok(var_name) => {
            let var_name_str = var_name.to_string();

            let output = quote! {
                {
                    // Get the type of the variable using std::any::type_name
                    let type_name = std::any::type_name_of_val(&#var_name);

                    // For simplicity, convert to string (in a real implementation,
                    // this would be more sophisticated)
                    let value_str = format!("{:?}", #var_name);

                    // Check if the variable is mutable - this is a simplified approach
                    // In a full implementation, would need more complex analysis
                    let is_mutable = false;

                    ::dbug::_internal::register_variable(#var_name_str, type_name, &value_str, is_mutable);
                }
            };

            output.into()
        }
        Err(err) => {
            // Output a compile error if we fail to parse
            let error_message = format!("Error parsing variable name in register_var!: {}", err);
            let error = quote::quote! {
                compile_error!(#error_message);
            };
            error.into()
        }
    }
}

/// Marks an async function for debugging with dbug
///
/// This macro processes an async function and inserts instrumentation code
/// to enable debugging with proper support for async execution.
///
/// # Example
///
/// ```
/// use dbug::prelude::*;
///
/// #[dbug_async]
/// async fn my_async_function(x: i32) -> i32 {
///     let y = x * 2;
///     dbug::break_here!();  // A debug point
///     y + 10
/// }
/// ```
#[proc_macro_attribute]
pub fn dbug_async(_attr: TokenStream, item: TokenStream) -> TokenStream {
    // Parse the input function
    let mut input_fn = parse_macro_input!(item as ItemFn);

    // Check if the function is async
    let is_async = input_fn.sig.asyncness.is_some();

    if !is_async {
        // Return an error if the function is not async
        let error = syn::Error::new_spanned(
            &input_fn.sig,
            "dbug_async can only be used on async functions",
        );
        return TokenStream::from(error.to_compile_error());
    }

    // Get function details
    let fn_name = &input_fn.sig.ident;
    let fn_name_str = fn_name.to_string();

    // Add exit instrumentation using a guard pattern with Drop
    // to ensure it's called on all exit paths, including early returns and panics
    let block = &input_fn.block;

    let new_block: Block = parse_quote! {{
        // Create a guard struct to handle function exit
        struct _DbugAsyncGuard<'a> {
            fn_name: &'a str,
            task_id: u64,
        }

        impl<'a> Drop for _DbugAsyncGuard<'a> {
            fn drop(&mut self) {
                ::dbug::_internal::exit_async_function(self.fn_name, self.task_id);
            }
        }

        // Generate a unique task identifier for this async execution
        let task_id = ::dbug::_internal::generate_async_task_id();

        // Create the guard - will be dropped when the function exits
        let _guard = _DbugAsyncGuard {
            fn_name: #fn_name_str,
            task_id
        };

        // Notify function entry
        ::dbug::_internal::enter_async_function(#fn_name_str, task_id);

        // Original function body continues here
        // The async execution will be tracked by the task_id
        #block
    }};

    // Replace the function block with our instrumented block
    input_fn.block = Box::new(new_block);

    // Convert back to TokenStream
    let output = input_fn.to_token_stream();
    output.into()
}

/// A macro for marking an async breakpoint in code
///
/// # Example
///
/// ```
/// use dbug::prelude::*;
///
/// async fn my_async_function() {
///     let x = 42;
///     dbug::async_break_here!();  // This will pause execution when running with dbug
///     println!("After breakpoint");
/// }
/// ```
#[proc_macro]
pub fn async_break_here(_input: TokenStream) -> TokenStream {
    let output = quote! {
        {
            let task_id = ::dbug::_internal::get_current_async_task_id();
            ::dbug::_internal::async_break_point(file!(), line!(), column!(), task_id);
        }
    };

    output.into()
}

/// A macro for marking an async breakpoint in code with a condition
///
/// This is similar to async_break_here, but only breaks when the condition is true.
///
/// # Example
///
/// ```
/// use dbug::prelude::*;
///
/// async fn my_async_function(x: i32) {
///     for i in 0..10 {
///         dbug::async_break_when!(i > 5);  // This will pause execution only when i > 5
///         println!("i = {}", i);
///     }
/// }
/// ```
#[proc_macro]
pub fn async_break_when(input: TokenStream) -> TokenStream {
    let condition = parse_macro_input!(input as Expr);

    let output = quote! {
        {
            let task_id = ::dbug::_internal::get_current_async_task_id();
            if #condition {
                ::dbug::_internal::async_break_point(file!(), line!(), column!(), task_id);
            }
        }
    };

    output.into()
}
