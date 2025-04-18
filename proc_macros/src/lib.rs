use proc_macro::TokenStream;
use quote::{quote, ToTokens};
use syn::{parse_macro_input, ItemFn, Stmt, parse_quote};

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
    let mut input_fn = parse_macro_input!(item as ItemFn);
    
    // Insert instrumentation at the beginning of the function
    let fn_name = &input_fn.sig.ident;
    let fn_name_str = fn_name.to_string();
    
    let entry_instrumentation: Stmt = parse_quote! {
        ::dbug::_internal::enter_function(#fn_name_str);
    };
    
    // We're careful not to add the exit instrumentation at the end, which would cause
    // the function to return () instead of its normal return type
    
    // Insert entry at the beginning
    input_fn.block.stmts.insert(0, entry_instrumentation);
    
    // Convert back to TokenStream
    input_fn.to_token_stream().into()
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
    let item_copy = item.clone();
    let item_ast = parse_macro_input!(item_copy as Stmt);
    
    let output = quote! {
        {
            ::dbug::_internal::break_point(file!(), line!(), column!());
            #item_ast
        }
    };
    
    output.into()
} 