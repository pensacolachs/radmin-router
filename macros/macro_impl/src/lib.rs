use syn::punctuated::Punctuated;
use quote::{quote, quote_spanned, ToTokens};
use syn::spanned::Spanned;
use syn::{parse_macro_input, parse_quote, parse_quote_spanned, Data, DeriveInput, Expr, Fields, ItemFn, ReturnType, Stmt, Token};

#[proc_macro_attribute]
pub fn box_future(_args: proc_macro::TokenStream, input: proc_macro::TokenStream) -> proc_macro::TokenStream {
    let input = parse_macro_input!(input as ItemFn);
    let ItemFn { attrs, vis, mut sig, block } = input;

    match sig.asyncness {
        Some(_) => {
            sig.asyncness = None;
        }

        None => {
            return syn::Error::new(
                sig.asyncness.span(),
                "expected async function")
                .into_compile_error()
                .into()
        }
    }

    let ret = match &sig.output {
        ReturnType::Default => quote_spanned!(sig.paren_token.span=> ()),
        ReturnType::Type(_, ret) => quote!(#ret)
    };
    sig.output = parse_quote_spanned!(ret.span()=>
        -> ::std::pin::Pin<::std::boxed::Box<dyn ::std::future::Future<Output = #ret>>>
    );

    let expanded = quote! {
        #(#attrs )* #vis #sig {
            ::std::boxed::Box::pin(async #block)
        }
    };

    proc_macro::TokenStream::from(expanded)
}

#[proc_macro_derive(CaseIterable)]
pub fn derive_case_iterable(input: proc_macro::TokenStream) -> proc_macro::TokenStream {
    let input = parse_macro_input!(input as DeriveInput);

    let ident = &input.ident;

    let Data::Enum(ref data) = input.data else {
        return syn::Error::new(input.span(), "only enums can be case-iterable")
            .into_compile_error()
            .into()
    };

    let mut cases = Punctuated::<Expr, Token![,]>::new();

    for var in &data.variants {
        let Fields::Unit = var.fields else {
            return syn::Error::new(var.fields.span(), "case-iterable enum variants cannot have fields")
                .into_compile_error()
                .into()
        };

        let ident = &var.ident;
        cases.push(parse_quote! { Self::#ident });
    }

    let cases_len = cases.len();

    let expanded = quote! {
        impl #ident {
            const fn all_cases() -> [Self; #cases_len] {
                [#cases]
            }
        }

        impl CaseIterable for #ident {
            const ALL_CASES: &'static [Self] = &[#cases];
        }
    };

    expanded.into()
}