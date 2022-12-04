use proc_macro::TokenStream;
use proc_macro2::Span;
use quote::ToTokens;
use quote::format_ident;
use syn::parse::Parse;
use syn::parse_macro_input;
use syn::Item;
use quote::quote;

#[proc_macro_attribute]
pub fn bitfield(args: TokenStream, input: TokenStream) -> TokenStream {
    let _ = args;
    // let _ = input;
    let item = parse_macro_input!(input as Item);
    // let ts = item.to_token_stream();
    let r = impl_bitfield(&item).unwrap();
    // println!("ts: {}", ts);
    // println!("r: {}", r);

    // unimplemented!()
    r.into()
}

fn impl_bitfield(item: &Item) -> Result<proc_macro2::TokenStream, ()> {
    if let syn::Item::Struct(s) = item {
        println!("is struct");
        let mut size = quote!(0);
        let mut field_attrs = vec![];
        for field in  s.fields.iter() {
            let ident = field.ident.as_ref().unwrap();  // named field only
            let ty = &field.ty;
            let item_ty = quote! { <#ty as ::bitfield::Specifier>::T };
            let bits = quote! { <#ty as ::bitfield::Specifier>::BITS };
            let sz = quote! { + #bits };
            // let sz = quote! { + <#ty as bitfield::Specifier>::BITS };
            println!("sz: {}", sz);
            field_attrs.push((ident, item_ty, bits, size.clone()));
            size.extend(sz);
        }

        // Rewrite layout.
        let size_token = quote! { (#size) / 8 };
        println!("size_token: {}", size_token);
        let ident = &s.ident;
        let vis = &s.vis;
        let mut output = proc_macro2::TokenStream::new();
        output.extend(quote! {
            #vis struct #ident {
                data: [u8; #size_token],
            }

            impl #ident {
                #vis fn new() -> Self {
                    #ident {
                        data: [0; #size_token],
                    } 
                }
            }
        });
        // let newfn = quote! {
        //     impl #ident {
        //         #vis fn new() -> Self {
        //             data: [0; {#size_token}],
        //         }
        //     }
        // };
        // println!("newfn: {}", newfn);

        // Get & Set methods.
        let total = quote! { (#size) };
        let mut getset = proc_macro2::TokenStream::new();
        for (ident, ty, bits, offset) in &field_attrs {
            // println!("ident: {}, ty: {}, bits: {}, offset: {}", ident, ty, bits, offset);
            let set_ident = format_ident!("set_{}", ident.to_string());
            let get_ident = format_ident!("get_{}", ident.to_string());
            let temp = quote! {
                pub fn #set_ident(&mut self, x: #ty) {
                    // <#ty as ::bitfield::Access<#bits, 0, 8>>::SET(&mut self.data, x);
                    <#ty as ::bitfield::Access<{#bits}, {#offset}, {#total}>>::SET(&mut self.data, x);
                }
                pub fn #get_ident(&self) -> #ty {
                    <#ty as ::bitfield::Access<{#bits}, {#offset}, {#total}>>::GET(&self.data)
                }
            };
            // println!("temp: {}", temp);
            getset.extend(temp);
        }

        output.extend(quote! {
            impl #ident {
                #getset
            }
        });

        println!(">>> output:\n{}", output);

        return Ok(output);
    }
    Err(())
}

#[proc_macro]
pub fn specifier(input: TokenStream) -> TokenStream {
    let si = parse_macro_input!(input as SpecifierInput);
    println!("parsed input maxn={}", si.maxn);
    let mut output = proc_macro2::TokenStream::new();
    for n in 1..=si.maxn {
        output.extend(bits_define(n));
    }
    // let expand = bits_define(1);
    // expand.into()
    output.into()
}

struct SpecifierInput {
    maxn: usize,
}

impl Parse for SpecifierInput {
    fn parse(input: syn::parse::ParseStream) -> syn::Result<Self> {
        let maxn: syn::LitInt = input.parse()?;
        Ok(SpecifierInput {
            maxn: maxn.base10_parse()?,
        })
    }
}

fn bits_define(n: usize) -> proc_macro2::TokenStream {
    let ident_string = format!("B{}", n);
    let ident = syn::Ident::new(&ident_string, Span::call_site());
    let bits = syn::LitInt::new(&format!("{}usize", n), Span::call_site());
    let ty = bits_type(n);
    quote! {
        pub enum #ident {}
        impl Specifier for #ident {
            const BITS: usize = #bits;
            type T = #ty;
        }
    }
}

fn bits_type(n: usize) -> proc_macro2::TokenStream {
    if n <= 8 {
        quote!(u8)
    } else if n <= 16 {
        quote!(u16)
    } else if n <= 32 {
        quote!(u32)
    } else if n <= 64 {
        quote!(u64)
    } else {
        panic!("Not support bits > 64.")
    }
}