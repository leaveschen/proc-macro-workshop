use proc_macro::TokenStream;
use syn::{parse_macro_input, parse_quote, DeriveInput, Data};
use quote::{quote, ToTokens};

#[proc_macro_derive(CustomDebug, attributes(debug))]
pub fn derive(input: TokenStream) -> TokenStream {
    let ast = parse_macro_input!(input as DeriveInput);

    let out = match derive_custom_debug(&ast) {
        Ok(out) => out.into(),
        Err(e) => e.to_compile_error(),
    };
    out.into()
}

fn derive_custom_debug(ast: &DeriveInput) -> Result<proc_macro2::TokenStream, syn::Error> {
    let ident = &ast.ident;
    let attrs = &ast.attrs;
    // for attr in attrs {
    //     eprintln!("[Struct attr]: {}", attr.to_token_stream());
    // }
    let escape_hatch = escape_hatch_attr(attrs);
    let generics = if let Some(escape_hatch) = &escape_hatch {
        add_trait_bound_with_escape_hatch(ast.generics.clone(), &escape_hatch)
    } else {
        add_trait_bound(ast.generics.clone(), &ast.data)
    };

    let (impl_generics, ty_generics, where_clause) = generics.split_for_impl();

    // if let Data::Struct(ds) = &ast.data {
    //     let fields_iter = ds.fields.iter()
    //         .map(|f| (&f.attrs, f.ident.as_ref().unwrap(), &f.ty));

    //     let _ = fields_iter
    //         .map(|(attrs, _, _)| fmt_attrs(attrs))
    //         .collect::<Vec<_>>();
    // }

    let fields_token = match &ast.data {
        Data::Struct(s) => {
            let fields_token = s.fields.iter()
                .map(|f| fmt_field(f));
            proc_macro2::TokenStream::from_iter(fields_token)
        },
        _ => {
            return Err(syn::Error::new_spanned(ident, "Only struct implemented"));
        }
    };

    let out = quote! {
        impl #impl_generics std::fmt::Debug for #ident #ty_generics #where_clause {
            fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
                f.debug_struct(stringify!(#ident))
                    #fields_token
                    .finish()
            }
        }
    };

    eprintln!(">>> out: {}", out);
    Ok(out)
}


fn fmt_field(f: &syn::Field) -> proc_macro2::TokenStream {
    let ident = f.ident.as_ref().unwrap();
    match fmt_attrs(&f.attrs) {
        Some(lit) => {
            let fmt = lit.to_token_stream();
            quote! {
                .field(stringify!(#ident), &::core::format_args!(#fmt, &self.#ident))
            }
        }
        None => quote! {
            .field(stringify!(#ident), &self.#ident)
        }
    }
}

fn fmt_attrs(attrs: &Vec<syn::Attribute>) -> Option<syn::Lit> {
    if let &[attr] = &attrs.as_slice() {
        if let Ok(syn::Meta::NameValue(nv)) = attr.parse_meta() {
            if nv.path.is_ident("debug") {
                return Some(nv.lit);
            }
        }
    }
    None
}

fn add_trait_bound(mut generics: syn::Generics, data: &syn::Data) -> syn::Generics {
    let mut associated_types: Vec<proc_macro2::TokenStream> = vec![];
    for p in &mut generics.params {
        if let syn::GenericParam::Type(t) = p {
            // eprintln!("[Generic Type] {}", t.to_token_stream());
            // eprintln!("[Generic Type ident] {}", t.ident.to_string());
            let skip = skip_trait_bound(t, data, &mut associated_types);
            // eprintln!(">>> skip trait bound: {}", skip);
            if !skip {
                t.bounds.push(parse_quote!(std::fmt::Debug));
            }
        }
    }

    // add associated types bound into where clause
    // for t in &associated_types {
    //     eprintln!("[Generic associated type] {}", t);
    // }
    generics.make_where_clause();
    let where_clause = generics.where_clause.as_mut().unwrap();
    for t in &associated_types {
        where_clause.predicates.push(parse_quote!(#t : std::fmt::Debug));
    }
    generics
}

// Check whether the type parameter `T` should escape the trait bound of `std::fmt::Debug`.
fn skip_trait_bound(type_param: &syn::TypeParam, data: &syn::Data,
                    associated_types: &mut Vec<proc_macro2::TokenStream>) -> bool {
    match &data {
        Data::Struct(ds) => {
            // let type_param_string = type_param.to_token_stream().to_string();
            let type_param_string = type_param.ident.to_string();
            ds.fields.iter()
                .all(|field| {
                    let mut most_inner_types = MostInnerTypes::new(&field.ty, &type_param_string);
                    associated_types.append(&mut most_inner_types.extra);
                    !most_inner_types.result.contains(&type_param_string)
                })
        },
        _ => unimplemented!()
    }
}

struct MostInnerTypes {
    pub result: Vec<String>,
    pub extra: Vec<proc_macro2::TokenStream>,
}

impl MostInnerTypes {
    fn new(ty: &syn::Type, param_string: &str) -> Self {
        let ty = ty.clone();
        // eprintln!("[Parse type]: {}", ty.to_token_stream());
        let mut result: Vec<String> = vec![];
        let mut extra: Vec<proc_macro2::TokenStream> = vec![];
        Self::traverse(&mut result, &mut extra, &ty, &param_string);
        // eprintln!("[Inner types]: {:?}", result);
        // eprintln!("[Extra types]: {:?}", extra);
        MostInnerTypes { result, extra }
    }

    fn traverse(result: &mut Vec<String>, extra: &mut Vec<proc_macro2::TokenStream>,
                node: &syn::Type, param_string: &str) {
        if let syn::Type::Path(syn::TypePath {
            qself: None,
            path: syn::Path { leading_colon: None, segments},
        }) = node {
            // eprintln!(">>> traverse node: {}", segments.to_token_stream());
            // eprintln!(">>> segment len: {}", segments.len());
            if let Some(segment) = segments.first() {
                // ignore `PhantomData`
                let ident = segment.ident.to_string();
                if ident.as_str() == "PhantomData" {
                    return;
                }
                // traverse stop
                if let syn::PathArguments::None = segment.arguments {
                    if segments.len() > 1 && ident.as_str() == param_string {
                        // extra.push(segments.to_token_stream().to_string());
                        extra.push(segments.to_token_stream());
                    } else {
                        result.push(ident);
                    }
                    return;
                }
                // generics
                // eprintln!(">>> {}", segment.arguments.to_token_stream());
                if let syn::PathArguments::AngleBracketed(args) = &segment.arguments {
                    for arg in args.args.iter() {
                        if let syn::GenericArgument::Type(next) = arg {
                            Self::traverse(result, extra, next, param_string);
                        }
                    }
                }
            }
        }
    }
}

fn escape_hatch_attr(attrs: &Vec<syn::Attribute>) -> Option<String>{
    if let [attr] = attrs.as_slice() {
        if let Ok(syn::Meta::List(list)) = attr.parse_meta() {
            // eprintln!("[Sturct meta] is list {}", list.to_token_stream());
            // eprintln!(">>> path: {}", list.path.to_token_stream());
            if list.path.is_ident("debug") {
                if let [syn::NestedMeta::Meta(nmeta)] = list.nested.iter().collect::<Vec<_>>().as_slice() {
                    // eprintln!(">>> meta: {}", nmeta.to_token_stream());
                    if let syn::Meta::NameValue(nv) = nmeta {
                        if nv.path.is_ident("bound") {
                            if let syn::Lit::Str(lit) = &nv.lit {
                                return Some(lit.value())
                            }
                        }
                        // eprintln!(">>> {}, {}", nv.path.to_token_stream(), nv.lit.to_token_stream());
                    }
                }
            }
        }
    }
    None
}

fn add_trait_bound_with_escape_hatch(mut generics: syn::Generics, escape_hatch: &str) -> syn::Generics {
    generics.make_where_clause();
    let where_clause = generics.where_clause.as_mut().unwrap();
    // eprintln!(">>> hatch: {}", escape_hatch.to_token_stream());
    where_clause.predicates.push(syn::parse_str(escape_hatch).unwrap());
    generics
}
