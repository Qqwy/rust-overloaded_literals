#![doc = include_str!("../README.md")]
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
        }
        ExprLit {
            attrs,
            lit: Lit::Float(lit_float),
        } => {
            if !attrs.is_empty() {
                return None;
            }
            let float = lit_float.base10_parse::<f64>().unwrap();
            let float_bits: u64 = (-float).to_bits();
            let res = parse_quote_spanned!(span=> ::overloaded_literals::FromLiteralFloat::<::overloaded_literals::type_float::Float<#float_bits>>::into_self());
            Some(res)
        }

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
        }
        ExprLit {
            attrs,
            lit: Lit::Bool(_),
        } => {
            if !attrs.is_empty() {
                return Expr::Lit(expr_lit);
            }
            let res = parse_quote_spanned!(span=> ::overloaded_literals::FromLiteralBool::<#expr_lit>::into_self());
            res
        }
        ExprLit {
            attrs,
            lit: Lit::Float(lit_float),
        } => {
            if !attrs.is_empty() {
                return Expr::Lit(expr_lit);
            }
            let float_bits: u64 = lit_float.base10_parse::<f64>().unwrap().to_bits();
            let res = parse_quote_spanned!(span=> ::overloaded_literals::FromLiteralFloat::<::overloaded_literals::type_float::Float<#float_bits>>::into_self());
            res
        }
        other => Expr::Lit(other.clone()),
    }
}

fn build_typestr(string: &str, span: Span) -> syn::Expr {
    let mut res = quote!(::tlist::TNil);
    for byte in string.as_bytes().iter().rev() {
        res = parse_quote_spanned!(span=> ::tlist::TCons<::overloaded_literals::type_str::Byte<#byte>, #res>);
    }
    let res =
        parse_quote_spanned!(span=> ::overloaded_literals::FromLiteralStr::<#res>::into_self());
    res
}

impl Fold for Args {
    // We fold at the level of `Expr` because when we change a literal, the result will be an `Expr`.
    fn fold_expr(&mut self, expr: syn::Expr) -> syn::Expr {
        // Needed since we want to traverse bottom-up and leave all other nodes intact:
        // let expr = syn::fold::fold_expr(self, expr);

        let span = expr.span();
        match expr {
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
                    let expr = Box::new(self.fold_expr(*boxed_expr));
                    Expr::Unary(ExprUnary { attrs, op, expr })
                }
            },
            Expr::Lit(expr_lit) => {
                // Positive int or string literals are 'plain' Expr::Lit
                wrap_unsigned_or_str(expr_lit, span)
            }
            other => syn::fold::fold_expr(self, other),
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
