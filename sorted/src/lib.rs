use proc_macro::TokenStream;
use syn::{parse_macro_input, Item, ItemFn, spanned::Spanned};
use syn::visit_mut::VisitMut;
use quote::quote;


#[proc_macro_attribute]
pub fn check(_args: TokenStream, input: TokenStream) -> TokenStream {
    println!("[macro check]");
    let mut ast = parse_macro_input!(input as ItemFn);
    let mut sorted_match = SortedMatch { err: None };
    sorted_match.visit_item_fn_mut(&mut ast);

    let mut output = proc_macro2::TokenStream::new();
    output.extend(quote!(#ast));
    if let Some(e) = &sorted_match.err {
        let e = e.to_compile_error();
        output.extend(quote!(#e));
    }
    output.into()
}

struct SortedMatch {
    err: Option<syn::Error>,
}

impl VisitMut for SortedMatch {
    fn visit_expr_match_mut(&mut self, i: &mut syn::ExprMatch) {
        println!(">>> find expr match: {}", quote!(#i));
        if self.is_marked_sorted(i) {
            self.check_order(i);
        }

        // Delegate to the default impl to visit nested expressions.
        syn::visit_mut::visit_expr_match_mut(self, i);
    }
}

impl SortedMatch {
    fn is_marked_sorted(&self, i: &mut syn::ExprMatch) -> bool {
        for (idx, attr) in i.attrs.iter().enumerate() {
            if let syn::Path { leading_colon: None, segments } = &attr.path {
                if let &[p] = segments.iter().collect::<Vec<_>>().as_slice() {
                    if p.ident.to_string().as_str() == "sorted" {
                        println!(">>> attr: {}", quote!(#p));
                        // Remove the `sorted` attribute.
                        i.attrs.remove(idx);
                        return true;
                    }
                }
            }
        }
        false
    }

    fn check_order(&mut self, i: &mut syn::ExprMatch) {
        let mut arms = vec![];
        for arm in i.arms.iter() {
            // println!("arm is {}", quote!(#arm));
            let pat = &arm.pat;
            if let syn::Pat::TupleStruct(t) = pat {
                // Pattern ident as `String`.
                let path = &t.path;
                let ts = quote!(#path).to_string().replace(" ", "");
                arms.push((OutOfOrderErr::Path(path), ts));
                // arms.push((path, ts));
            } else if let syn::Pat::Ident(i) = pat {
                let ts = i.ident.to_string();
                // println!("arm is Ident: {}", ts);
                arms.push((OutOfOrderErr::Ident(&i.ident), ts));
            } else if let syn::Pat::Wild(w) = pat {
                w.underscore_token;
                let ts = quote!(#w).to_string();
                // println!("arm is Wild: {}", ts);
                arms.push((OutOfOrderErr::Wild(w), ts));
            } else {
                println!("[unrecognized pattern]: {}", quote!(#pat));
                self.err = Some(syn::Error::new_spanned(pat, "unsupported by #[sorted]"));
                return;
            }
        }

        for i in 0..arms.len() {
            for j in i+1..arms.len() {
                let xs = arms[i].1.as_str();
                let ys = arms[j].1.as_str();
                if ys != "_" && xs > ys {
                    println!("[out of order]: {} > {}", xs, ys);
                    self.err = Some(arms[j].0.error(xs, ys));
                    return;
                }
            }
        }
    }
}


enum OutOfOrderErr<'a> {
    Path(&'a syn::Path),
    Ident(&'a syn::Ident),
    Wild(&'a syn::PatWild),
}

impl<'a> OutOfOrderErr<'a> {
    fn error(&self, xs: &str, ys: &str) -> syn::Error {
        let msg = format!("{} should sort before {}", ys, xs);
        match &self {
            Self::Path(p) => syn::Error::new_spanned(p, msg),
            Self::Ident(i) => syn::Error::new_spanned(i, msg),
            Self::Wild(w) => syn::Error::new_spanned(w, msg),
        }
    }
}

/* ------------------------------------------------------ */

#[proc_macro_attribute]
pub fn sorted(_args: TokenStream, input: TokenStream) -> TokenStream {
    // println!("args: {}", args);
    println!("input: {}", input);
    let item = parse_macro_input!(input as Item);

    let s = sorted_impl(&item);
    s.token_stream().into()
}

struct Sorted {
    out: proc_macro2::TokenStream,
    err: Option<syn::Error>,
}

impl Sorted {
    fn token_stream(&self) -> proc_macro2::TokenStream {
        let out = &self.out;
        let err = self.err.as_ref().map(|e| e.clone().into_compile_error());
        match err {
            Some(e) => quote! {
                #out
                #e
            },
            None => quote!{#out},
        }
    }
}

fn sorted_impl(item: &Item) -> Sorted {
    let mut out = proc_macro2::TokenStream::new();
    let err;
    if let Item::Enum(item_enum) = item {
        out.extend(quote!{#item_enum});
        err = check_order(item_enum);
    } else {
        err = Some(syn::Error::new(proc_macro2::Span::call_site(), "expected enum or match expression"));
    }
    Sorted { out, err }
}

fn check_order(item_enum: &syn::ItemEnum) -> Option<syn::Error> {
    let vars = item_enum.variants.iter().collect::<Vec<_>>();
    for i in 0..vars.len() {
        for j in i+1..vars.len() {
            let x = vars[i];
            let y = vars[j];
            // println!("compare {} to {}", x.ident.to_string(), y.ident.to_string());
            let xs = x.ident.to_string();
            let ys = y.ident.to_string();
            if xs > ys {
                println!(">>> out of order");
                let msg = format!("{} should sort before {}", ys, xs);
                let e = syn::Error::new(y.span(), msg);
                return Some(e);
            }
        }
    }
    None
}