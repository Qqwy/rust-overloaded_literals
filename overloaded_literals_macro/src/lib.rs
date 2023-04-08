extern crate proc_macro;
use proc_macro::TokenStream;
use quote::{__private::Span, quote};
use syn::{
    fold::Fold, parse_macro_input, parse_quote_spanned, spanned::Spanned, Expr, ExprLit, ExprUnary,
    ItemFn, Lit, UnOp,
};

struct Args;

fn wrap_signed(unsigned_expr_lit: &ExprLit, span: Span) -> Option<syn::Expr> {
    match unsigned_expr_lit {
        ExprLit {
            attrs,
            lit: Lit::Int(lit_int),
        } => {
            if !attrs.is_empty() {
                return None;
            }
            let res = parse_quote_spanned!(span=> ::overloaded_literals::FromLiteralSigned::<-#lit_int>::into_self() );
            Some(res)
        },
        // ExprLit {
        //     attrs,
        //     lit: Lit::Float(lit_float),
        // } => {
        //     if !attrs.is_empty() {
        //         return None;
        //     }
        //     let res = parse_quote_spanned!(span=> ::overloaded_literals::FromLiteralFloat::<-#lit_float>::into_self() );
        //     Some(res)
        // }
        _ => None,
    }
}

// NOTE: Make sure this value is not larger than the one in `overloaded_literals_macro`
const MAX_STR_LIT_LEN: usize = 32768;

fn wrap_unsigned_or_str(expr_lit: ExprLit, span: Span) -> syn::Expr {
    match &expr_lit {
        ExprLit {
            attrs,
            lit: Lit::Int(_lit_int),
        } => {
            if !attrs.is_empty() {
                return Expr::Lit(expr_lit);
            }
            let res = parse_quote_spanned!(span=> ::overloaded_literals::FromLiteralUnsigned::<#expr_lit>::into_self());
            res
        }
        ExprLit {
            attrs,
            lit: Lit::Str(lit_str),
        } => {
            if !attrs.is_empty() {
                return Expr::Lit(expr_lit);
            }
            if lit_str.value().len() > MAX_STR_LIT_LEN {
                return Expr::Lit(expr_lit);
            }
            build_typestr(&lit_str.value(), span)
        },
        ExprLit {
            attrs,
            lit: Lit::Bool(_),
        } => {
            if !attrs.is_empty() {
                return Expr::Lit(expr_lit);
            }
            let res = parse_quote_spanned!(span=> ::overloaded_literals::FromLiteralBool::<#expr_lit>::into_self());
            res
        },
        //     res
        // },
        // ExprLit {
        //     attrs,
        //     lit: Lit::Float(_lit_float),
        // } => {
        //     if !attrs.is_empty() {
        //         return Expr::Lit(expr_lit);
        //     }
        //     let res = parse_quote_spanned!(span=> ::overloaded_literals::FromLiteralFloat::<#expr_lit>::into_self());
        //     res
        // },
        other => Expr::Lit(other.clone()),
    }
}

fn build_typestr(string: &str, span: Span) -> syn::Expr {
    let mut res = quote!(::tlist::TNil);
    for byte in string.as_bytes().iter().rev() {
        res = parse_quote_spanned!(span=> ::tlist::TCons<::overloaded_literals::type_str::Byte<#byte>, #res>);
    }
    let res = parse_quote_spanned!(span=> ::overloaded_literals::FromLiteralStr::<#res>::into_self());
    res
}

impl Fold for Args {
    // We fold at the level of `Expr` because when we change a literal, the result will be an `Expr`.
    fn fold_expr(&mut self, i: syn::Expr) -> syn::Expr {
        let span = i.span();
        let f = self;
        match i {
            // Negative int literals are represented as Expr::Unary(UnOp::Neg, Expr::Lit(...))
            Expr::Unary(ExprUnary {
                attrs,
                op: op @ UnOp::Neg(_),
                expr: boxed_expr,
            }) => match &*boxed_expr {
                Expr::Lit(expr_lit) => wrap_signed(expr_lit, span).unwrap_or_else(|| {
                    Expr::Unary(ExprUnary {
                        attrs,
                        op,
                        expr: boxed_expr,
                    })
                }),
                _ => {
                    let expr = Box::new(f.fold_expr(*boxed_expr));
                    Expr::Unary(ExprUnary { attrs, op, expr })
                }
            },
            Expr::Lit(expr_lit) => {
                // Positive int or string literals are 'plain' Expr::Lit
                wrap_unsigned_or_str(expr_lit, span)
            },
            // We do not change any other type of expression,
            // but need to recurse into them.
            //
            // These clauses are copied over from the default implementation of `Fold::fold_expr`:
            Expr::Array(_binding_0) => Expr::Array(f.fold_expr_array(_binding_0)),
            Expr::Assign(_binding_0) => Expr::Assign(f.fold_expr_assign(_binding_0)),
            Expr::Async(_binding_0) => Expr::Async(f.fold_expr_async(_binding_0)),
            Expr::Await(_binding_0) => Expr::Await(f.fold_expr_await(_binding_0)),
            Expr::Binary(_binding_0) => Expr::Binary(f.fold_expr_binary(_binding_0)),
            Expr::Block(_binding_0) => Expr::Block(f.fold_expr_block(_binding_0)),
            Expr::Break(_binding_0) => Expr::Break(f.fold_expr_break(_binding_0)),
            Expr::Call(_binding_0) => Expr::Call(f.fold_expr_call(_binding_0)),
            Expr::Cast(_binding_0) => Expr::Cast(f.fold_expr_cast(_binding_0)),
            Expr::Closure(_binding_0) => Expr::Closure(f.fold_expr_closure(_binding_0)),
            Expr::Const(_binding_0) => Expr::Const(f.fold_expr_const(_binding_0)),
            Expr::Continue(_binding_0) => Expr::Continue(f.fold_expr_continue(_binding_0)),
            Expr::Field(_binding_0) => Expr::Field(f.fold_expr_field(_binding_0)),
            Expr::ForLoop(_binding_0) => Expr::ForLoop(f.fold_expr_for_loop(_binding_0)),
            Expr::Group(_binding_0) => Expr::Group(f.fold_expr_group(_binding_0)),
            Expr::If(_binding_0) => Expr::If(f.fold_expr_if(_binding_0)),
            Expr::Index(_binding_0) => Expr::Index(f.fold_expr_index(_binding_0)),
            Expr::Infer(_binding_0) => Expr::Infer(f.fold_expr_infer(_binding_0)),
            Expr::Let(_binding_0) => Expr::Let(f.fold_expr_let(_binding_0)),
            // Expr::Lit(_binding_0) => Expr::Lit(f.fold_expr_lit(_binding_0)), // <- This one we handle above
            Expr::Loop(_binding_0) => Expr::Loop(f.fold_expr_loop(_binding_0)),
            Expr::Macro(_binding_0) => Expr::Macro(f.fold_expr_macro(_binding_0)),
            Expr::Match(_binding_0) => Expr::Match(f.fold_expr_match(_binding_0)),
            Expr::MethodCall(_binding_0) => Expr::MethodCall(f.fold_expr_method_call(_binding_0)),
            Expr::Paren(_binding_0) => Expr::Paren(f.fold_expr_paren(_binding_0)),
            Expr::Path(_binding_0) => Expr::Path(f.fold_expr_path(_binding_0)),
            Expr::Range(_binding_0) => Expr::Range(f.fold_expr_range(_binding_0)),
            Expr::Reference(_binding_0) => Expr::Reference(f.fold_expr_reference(_binding_0)),
            Expr::Repeat(_binding_0) => Expr::Repeat(f.fold_expr_repeat(_binding_0)),
            Expr::Return(_binding_0) => Expr::Return(f.fold_expr_return(_binding_0)),
            Expr::Struct(_binding_0) => Expr::Struct(f.fold_expr_struct(_binding_0)),
            Expr::Try(_binding_0) => Expr::Try(f.fold_expr_try(_binding_0)),
            Expr::TryBlock(_binding_0) => Expr::TryBlock(f.fold_expr_try_block(_binding_0)),
            Expr::Tuple(_binding_0) => Expr::Tuple(f.fold_expr_tuple(_binding_0)),
            Expr::Unary(_binding_0) => Expr::Unary(f.fold_expr_unary(_binding_0)),
            Expr::Unsafe(_binding_0) => Expr::Unsafe(f.fold_expr_unsafe(_binding_0)),
            Expr::Verbatim(_binding_0) => Expr::Verbatim(_binding_0),
            Expr::While(_binding_0) => Expr::While(f.fold_expr_while(_binding_0)),
            Expr::Yield(_binding_0) => Expr::Yield(f.fold_expr_yield(_binding_0)),
            other => other,
        }
    }
}

#[proc_macro_attribute]
pub fn overloaded_literals(_metadata: TokenStream, input: TokenStream) -> TokenStream {
    let input_fn = parse_macro_input!(input as ItemFn);
    let mut args = Args;
    let output = args.fold_item_fn(input_fn);
    TokenStream::from(quote!(#output))
}


// These tests are mainly here for debugging;
// They (only) ensure the happy path does not crash.
// (And if it does, we have relatively easy debugging)
//
// More proper full-range tests can be found in the main crate.
#[cfg(test)]
mod tests {
    use syn::parse_quote;

    use super::*;

    #[test]
    fn unsigned_example() {
        let input_fun = parse_quote! {
            fn foo() {
                let res: u8 = foo(1, 1234567);
                res
            }
        };
        let mut args = Args;
        let _out = args.fold_item_fn(input_fun);
        // println!("{:?}", out)
    }

    #[test]
    fn signed_example() {
        let input_fun = parse_quote! {
            fn foo() {
                let res: u8 = bar(-10, -4200);
                res
            }
        };
        let mut args = Args;
        let _out = args.fold_item_fn(input_fun);
        // println!("{:?}", out)
    }

    #[test]
    fn string_example() {
        let input_fun = parse_quote! {
            fn foo() {
                let res: u8 = foo("bar", "baz");
                res
            }
        };
        let mut args = Args;
        let _out = args.fold_item_fn(input_fun);
        // println!("{:?}", out)
    }


    // #[test]
    // fn float_example() {
    //     let input_fun = parse_quote! {
    //         fn foo() {
    //             let res: u8 = foo(1.0, -42.0, 10e3.0);
    //             res
    //         }
    //     };
    //     let mut args = Args;
    //     let _out = args.fold_item_fn(input_fun);
    //     // println!("{:?}", out)
    // }

    #[test]
    fn mixed_example() {
        let input_fun = parse_quote! {
            fn foo() {
                let one: u8 = 1024;
                let two: String = "hello";
                let three: i8 = 20;
                let four : i8 = -33;
            }
        };
        let mut args = Args;
        let _out = args.fold_item_fn(input_fun);
        // println!("{:?}", out)
    }
}
