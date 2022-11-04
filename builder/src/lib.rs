use proc_macro::TokenStream;
use proc_macro2::Ident;
use syn::{parse_macro_input, DeriveInput, Type, Attribute, spanned::Spanned};
use quote::{quote, format_ident};



#[proc_macro_derive(Builder, attributes(builder))]
pub fn derive(input: TokenStream) -> TokenStream {
    let ast = parse_macro_input!(input as DeriveInput);

    let ts = match derive_builder(&ast) {
        Ok(ts) => ts,
        Err(e) => e.to_compile_error(),
    };

    // eprintln!("\n\n>>> out: \n{}", ts);
    ts.into()
}


fn derive_builder(ast: &DeriveInput) -> Result<proc_macro2::TokenStream, syn::Error> {
    let vis = &ast.vis;
    let ident = &ast.ident;
    let builder_ident = format_ident!("{}Builder", ident);

    match &ast.data {
        syn::Data::Struct(ds) => {
            let fields = &ds.fields;
            let fields_iter = fields.iter()
                .map(|f| (f.ident.as_ref().unwrap(), &f.ty, &f.attrs));

            // check correctness of attributes
            for (_, _, attrs) in fields_iter.clone() {
                if let Err(e) = extract_attr_value(attrs) {
                    return Err(e);
                }
            }

            // debug
            // let _s: Vec<_> = fields_iter.clone().map(|(fi, ty, _)| {
            //     eprintln!("--- field: {}", fi.to_string());
            //     // extract_attr_value(attrs);
            //     match ty {
            //         Type::Path(p) => {
            //             let seg = &p.path.segments;
            //             let s: Vec<_> = seg.iter().collect();
            //             for ss in s {
            //                 eprintln!("... seg: {}", ss.ident.to_string());
            //             }
            //         },
            //         _ => {},
            //     };
            //     eprintln!("");
            // }).collect();

            // builder field with extracted option
            let builder_fields = generate_map_fn(fields_iter.clone(), |(fi, ty, _)| {
                if let Some(inner_type) = get_option_type(ty) {
                    quote! { #fi: ::core::option::Option<#inner_type>, }
                } else if let Some(inner_type) = get_vec_type(ty) {
                    quote! { #fi: ::std::vec::Vec<#inner_type>, }
                } else {
                    quote! { #fi: ::core::option::Option<#ty>, }
                }
            });
            // eprintln!(">>> [fields]: {}", builder_fields);

            // initialize builder fields
            let builder_fields_init = generate_map_fn(fields_iter.clone(), |(fi, ty, _)| {
                if let Some(_) = get_vec_type(ty) {
                    quote! { #fi: ::std::vec::Vec::new(), }
                } else {
                    quote! { #fi: ::core::option::Option::None, }
                }
            });
            // eprintln!(">>> [fields init]: {}", builder_fields_init);

            // setter with extracted option
            let builder_setter = generate_map_fn(fields_iter.clone(), |(fi, ty, attrs)| {
                if let Some(inner_type) = get_option_type(ty) {
                    quote! {
                        #vis fn #fi(&mut self, #fi: #inner_type) -> &mut Self {
                            self.#fi = ::core::option::Option::Some(#fi);
                            self
                        }
                    }
                } else if let Some(_) = get_vec_type(ty) {
                    if is_attr_conflict(fi, attrs) {
                        quote!()
                    } else {
                        quote! {
                            #vis fn #fi(&mut self, mut #fi: #ty) -> &mut Self {
                                self.#fi.append(&mut #fi);
                                self
                            }
                        }
                    }
                } else {
                    quote! {
                        #vis fn #fi(&mut self, #fi: #ty) ->&mut Self {
                            self.#fi = ::core::option::Option::Some(#fi);
                            self
                        }
                    }
                }
            });
            // eprintln!(">>> [fields setter]: {}", builder_setter);

            let builder_repeated_setter = generate_map_fn(fields_iter.clone(), |(fi, ty, attrs)| {
                if let Some(inner_type) = get_vec_type(ty) {
                    if let Ok(Some(rident)) = extract_attr_value(attrs) {
                        quote! {
                            #vis fn #rident(&mut self, #rident: #inner_type) -> &mut Self {
                                self.#fi.push(#rident);
                                self
                            }
                        }
                    } else {
                        quote!()
                    }
                } else {
                    quote!()
                }
            });

            let builder_build_fields = generate_map_fn(fields_iter.clone(), |(fi, ty, _)| {
                if let Some(_) = get_option_type(ty) {
                    quote! { #fi: self.#fi.take(), }
                } else if let Some(_) = get_vec_type(ty) {
                    quote! { #fi: self.#fi.drain(0..).collect(), }
                } else {
                    quote! { #fi: self.#fi.take().unwrap(), }
                }
            });
            // eprintln!(">>> build fields: {}", builder_build_fields);

            let expand = quote! {
                #vis struct #builder_ident {
                    #builder_fields
                }

                impl #ident {
                    #vis fn builder() -> #builder_ident {
                        #builder_ident {
                            #builder_fields_init
                        }
                    }
                }

                impl #builder_ident {
                    #builder_setter
                    #builder_repeated_setter
                }

                impl #builder_ident {
                    #vis fn build(&mut self) -> ::core::result::Result<#ident, ()> {
                        let r = #ident {
                            #builder_build_fields
                        };
                        ::core::result::Result::Ok(r)
                    }
                }
            };
            Ok(expand)
        },
        _ => Err(syn::Error::new(ast.span(), "Only named struct allowed."))
    }
}

fn generate_map_fn<'a, F>(iter: impl Iterator<Item = (&'a Ident, &'a Type, &'a Vec<Attribute>)>, f: F) -> proc_macro2::TokenStream
where F: FnMut((&Ident, &Type, &Vec<Attribute>)) -> proc_macro2::TokenStream {
    proc_macro2::TokenStream::from_iter(iter.map(f))
}

fn generic_single_type(ty: &syn::Type) -> Option<(&syn::Ident, &syn::Type)> {
    match ty {
        // match segment path
        syn::Type::Path(syn::TypePath {
            qself: None,
            path: syn::Path { leading_colon: None, segments, },
        }) => {
            // only one segment for `Option<T>`
            if let [segment] = segments.iter().collect::<Vec<_>>().as_slice() {
                let ident = &segment.ident;  // generic type identifier
                // eprintln!("--- --- generic type is: {}", ident.to_string());
                // extract inner type
                if let syn::PathArguments::AngleBracketed(args) = &segment.arguments {
                    if let [syn::GenericArgument::Type(ty)] = args.args.iter().collect::<Vec<_>>().as_slice() {
                        // if let Type::Path(x) = ty {
                        //     let ss = x.path.segments.first().unwrap().ident.to_string();
                        //     eprintln!("--- --- --- inner type: {}", ss);
                        // }
                        return Some((ident, ty));
                    }
                }
            }
        },
        _ => {},
    }
    None
}

fn get_option_type(ty: &syn::Type) -> Option<&syn::Type> {
    generic_single_type(ty).and_then(|(ident, ty)| {
        if ident.to_string().as_str() == "Option" {
            Some(ty)
        } else {
            None
        }
    })
}

fn get_vec_type(ty: &syn::Type) -> Option<&syn::Type> {
    generic_single_type(ty).and_then(|(ident, ty)| {
        if ident.to_string().as_str() == "Vec" {
            Some(ty)
        } else {
            None
        }
    })
}

fn extract_attr_value(attrs: &Vec<Attribute>) -> Result<Option<syn::Ident>, syn::Error> {
    if let [attr] = attrs.as_slice() {
        // eprintln!("--- found attr: {}", attr.tokens);
        if let Ok(syn::Meta::List(list)) = attr.parse_meta() {
            if list.path.is_ident("builder") {
                // eprintln!("--- found attr: builder");
                if let [syn::NestedMeta::Meta(syn::Meta::NameValue(nv))] = list.nested.iter().collect::<Vec<_>>().as_slice() {
                    // eprintln!("--- attr matched single name value");
                    // eprintln!("{}", nv.path.get_ident().unwrap());
                    if nv.path.is_ident("each") {
                        if let syn::Lit::Str(s) = &nv.lit {
                            // eprintln!("value is {}", s.into_token_stream());
                            return Ok(Some(format_ident!("{}", s.value())));
                        }
                    } else {
                        let err = syn::Error::new_spanned(list, "expected `builder(each = \"...\")`");
                        return Err(err)
                    }
                }
            }
        }
    }
    Ok(None)
}

fn is_attr_conflict(field_ident: &syn::Ident, attrs: &Vec<Attribute>) -> bool {
    if let Ok(Some(attr_ident)) = extract_attr_value(attrs) {
        if attr_ident.to_string().as_str() == field_ident.to_string().as_str() {
            return true;
        }
    }
    false
}