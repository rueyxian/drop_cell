use proc_macro2::Span;
use quote::quote;
use syn::parenthesized;
use syn::parse_macro_input;
use syn::Token;

#[proc_macro]
pub fn defer(input: proc_macro::TokenStream) -> proc_macro::TokenStream {
    parse_macro_input!(input as Defer).0.into()
}

struct Defer(proc_macro2::TokenStream);

impl syn::parse::Parse for Defer {
    fn parse(input: syn::parse::ParseStream) -> syn::Result<Self> {
        let mut args = vec![];
        if input.peek(syn::token::Paren) {
            let content;
            parenthesized!(content in input);
            if !content.is_empty() {
                args.push(content.parse::<Arg>()?);
                while !content.is_empty() {
                    let _comma = content.parse::<Token![,]>()?;
                    args.push(content.parse::<Arg>()?);
                }
            }
            let _fat = input.parse::<Token![=>]>()?;
        } else {
            let fork = input.fork();
            if let Ok(_arg) = fork.parse::<Arg>() {
                if fork.peek(Token![=>]) {
                    args.push(input.parse::<Arg>()?);
                    let _fat = input.parse::<Token![=>]>()?;
                }
            }
        }
        let body = input.parse::<proc_macro2::TokenStream>()?;
        let cell = proc_macro2::Ident::new("__drop_cell", Span::mixed_site());
        let (maybe_mut, exprs, idents) = if args.is_empty() {
            (None, quote! {()}, quote! {_})
        } else if args.len() == 1 {
            let expr = &args[0].expr;
            let ident = &args[0].ident;
            (Some(quote! {mut}), quote! {#expr}, quote! {#ident})
        } else {
            let exprs = args.iter().map(|arg| &arg.expr);
            let idents = args.iter().map(|arg| &arg.ident);
            (
                Some(quote! {mut}),
                quote! {(#(#exprs,)*)},
                quote! {(#(#idents,)*)},
            )
        };
        let def_defer = quote! {
            let #maybe_mut #cell = drop_cell::cell::DropCell::new(#exprs, |#idents|{
                #body
            });
        };
        let def_muts = (!args.is_empty()).then(|| {
            quote! { let #idents = #cell.args_mut(); }
        });
        let tts = quote! {
            #def_defer
            #def_muts
        };
        Ok(Defer(tts))
    }
}

struct Arg {
    ident: syn::Ident,
    expr: proc_macro2::TokenStream,
}

impl syn::parse::Parse for Arg {
    fn parse(input: syn::parse::ParseStream) -> syn::Result<Self> {
        let (ident, expr) = if input.peek2(Token![@]) {
            let ident = input.parse::<syn::Ident>()?;
            let _at = input.parse::<Token![@]>()?;
            let expr = {
                let expr = input.parse::<syn::Expr>()?;
                quote! { #expr }
            };
            (ident, expr)
        } else {
            let ident = input.parse::<syn::Ident>()?;
            let expr = quote! { #ident };
            (ident, expr)
        };
        Ok(Arg { ident, expr })
    }
}
