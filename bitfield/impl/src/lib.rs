use proc_macro::TokenStream;
use proc_macro2::Span;
use quote::ToTokens;
use quote::format_ident;
use syn::parse::Parse;
use syn::parse_macro_input;
use syn::Item;
use syn::DeriveInput;
use quote::{quote, quote_spanned};

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
        let mut output = proc_macro2::TokenStream::new();
        let mut size = quote!(0);
        let mut field_attrs = vec![];
        for field in  s.fields.iter() {
            let ident = field.ident.as_ref().unwrap();  // named field only
            let ty = &field.ty;
            // For test 10 & 11.
            if let Some(bits) = parse_attr(&field.attrs) {
                let struct_ident = &s.ident;
                let scope = format_ident!("__attr_{}_{}", struct_ident.to_string(), ident.to_string());
                output.extend(quote_spanned!{ bits.span() =>
                    const fn #scope() {
                        struct Inner { x: [u8; #bits] }
                        const _: Inner = Inner { x: [0; <#ty as ::bitfield::Specifier>::BITS] };
                    }
                });
            }

            let item_ty = quote! { <#ty as ::bitfield::Specifier>::V };
            let inner_ty = quote! { <#ty as ::bitfield::Specifier>::T };
            let bits = quote! { <#ty as ::bitfield::Specifier>::BITS };
            let sz = quote! { + #bits };
            // let sz = quote! { + <#ty as bitfield::Specifier>::BITS };
            println!("sz: {}", sz);
            field_attrs.push((ident, item_ty, inner_ty, bits, size.clone()));
            size.extend(sz);
        }

        // Rewrite layout.
        let size_token = quote! { (#size) / 8 };
        println!("size_token: {}", size_token);
        let ident = &s.ident;
        let vis = &s.vis;
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

        /* For test-04
         * Because the proc macro expand before actual parse and aliasing,
         * it's impossiable the calculate the size during macro expansion.
         * Here I can't find any other ways to to the checking.
         * This code will satisfied the functionality of test-04, but the compile error 
         * generated do not match the error file given by the origin author.
         */
        output.extend(quote! {
            ::bitfield::static_assertions::const_assert_eq!((#size) % 8, 0);
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
        for (ident, item_ty, inner_ty, bits, offset) in &field_attrs {
            // println!("ident: {}, ty: {}, bits: {}, offset: {}", ident, ty, bits, offset);
            let set_ident = format_ident!("set_{}", ident.to_string());
            let get_ident = format_ident!("get_{}", ident.to_string());
            let temp = quote! {
                pub fn #set_ident(&mut self, x: #item_ty) {
                    // <#ty as ::bitfield::Access<#bits, 0, 8>>::SET(&mut self.data, x);
                    <#inner_ty as ::bitfield::Access<{#bits}, {#offset}, {#total}>>::SET(&mut self.data, x.binto());
                }
                pub fn #get_ident(&self) -> #item_ty {
                    <#inner_ty as ::bitfield::Access<{#bits}, {#offset}, {#total}>>::GET(&self.data).binto()
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

// For test 10 & 11.
fn parse_attr(attrs: &Vec<syn::Attribute>) -> Option<syn::Lit> {
    if let [attr] = attrs.as_slice() {
        println!("[--attr] {}", attr.tokens);
        if let Ok(syn::Meta::NameValue(nv)) = attr.parse_meta() {
            if nv.path.is_ident("bits") {
                println!("[--attr] is named value, with name=bits");
                println!("[--attr] value={}", nv.lit.to_token_stream());
                return Some(nv.lit.clone());
            }
        }
    }
    None
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
            type V = #ty;
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


/* Derive macro for building enum specifier */
#[proc_macro_derive(BitfieldSpecifier)]
pub fn derive_specifier(input: TokenStream) -> TokenStream {
    let ast = parse_macro_input!(input as DeriveInput);
    let mut output = proc_macro2::TokenStream::new();
    if let syn::Data::Enum(de) = &ast.data {
        let ident = &ast.ident;
        let num_arms = de.variants.iter().len();
        let bits = match perfect_log2(num_arms) {
            Ok(v) => v,
            // For test-08.
            Err(_) => {
                const MSG: &'static str = "BitfieldSpecifier expected a number of variants which is a power of 2";
                let err = syn::Error::new(Span::call_site(), MSG);
                return err.to_compile_error().into();
            },
        };
        let bits = syn::LitInt::new(&bits.to_string(), Span::call_site());
        let ty = bits_type(num_arms);

        output.extend(quote! {
            impl ::bitfield::Specifier for #ident {
                const BITS: usize = #bits;
                type T = #ty;
                type V = #ident;
            }
        });

        // Token stream of arms of convert Specifier::T to Specifier::V.
        let mut to_t = vec![];
        // Token stream of arms of convert Specifier::V to Specifier::T.
        let mut to_v = vec![];
        // Token stream of arms of discriminant values.
        let mut disc = vec![];

        // For test-09
        let mut check_range = vec![];
        let arms = syn::LitInt::new(&num_arms.to_string(), Span::call_site());

        for var in de.variants.iter() {
            println!("var: {}", var.to_token_stream());
            let v_ident = &var.ident;

            to_t.push(quote!{ #ident::#v_ident => #ident::#v_ident as #ty, });
            to_v.push(quote!{ #v_ident => #ident::#v_ident, });
            disc.push(quote!{ const #v_ident: #ty = #ident::#v_ident as #ty; });

            let check_range_const = quote_spanned!{ v_ident.span() =>
                #[allow(non_upper_case_globals)]
                const #v_ident: usize = #ident::#v_ident as usize;
                impl ::bitfield::check::CheckRangeTrait<<::bitfield::check::CheckRange::<#v_ident, {#arms > #v_ident}> as ::bitfield::check::Tag>::T> for Empty::<#v_ident> {}
            };
            check_range.push(check_range_const);
        }

        let to_t_iter = to_t.iter();
        let to_v_iter = to_v.iter();
        let disc_iter = disc.iter();
        // The first arm of target enum.
        // This unwrap will be safe because the perfect_log2 reject the empty enum case.
        let default_disc = &de.variants.first().unwrap().ident;
        let default = quote!( _ => #ident::#default_disc, );
        // For test-09.
        let check_range_iter = check_range.iter();
        let check_ident = format_ident!("__check_{}", ident.to_string());
        output.extend(quote!{
            impl ::bitfield::BInto<#ty> for #ident {
                fn binto(self) -> #ty {
                    match self {
                        #( #to_t_iter )*
                    }
                }
            }

            impl ::bitfield::BInto<#ident> for #ty {
                fn binto(self) -> #ident {
                    #![allow(non_upper_case_globals)]
                    #( #disc_iter )*
                    match self {
                        #( #to_v_iter )*
                        #default
                    }
                }
            }

            #[allow(non_snake_case)]
            const fn #check_ident() {
                struct Empty<const U: usize>;
                #( #check_range_iter )*
            }
        });
    }

    println!("derive output: {}", output);
    
    output.into()
}

// Function to calculate log2 for integer.
// If input value is 2^P result log2(P), otherwise return error.
const USIZE_BITS: u32 = (std::mem::size_of::<usize>() * 8) as u32;
fn perfect_log2(x: usize) -> Result<u32, ()> {
    if x.count_ones() != 1 {
        return Err(());
    }
    Ok(USIZE_BITS - x.leading_zeros() -1)
}