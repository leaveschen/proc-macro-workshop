use proc_macro::TokenStream;
use proc_macro2::TokenTree;
use proc_macro2::Delimiter;
use syn::parse::{Parse, ParseStream};
use syn::{Ident, Token, LitInt, braced};
use syn::parse_macro_input;
use syn::buffer::{TokenBuffer, Cursor};
use quote::quote;

struct SeqInput {
    ident: Ident,
    // in_token: syn::Token![in],
    begin: isize,
    // range_token: syn::Token![..],
    end: isize,
    content: proc_macro2::TokenStream,
}

impl Parse for SeqInput {
    fn parse(input: ParseStream) -> syn::Result<Self> {
        let ident: Ident = input.parse()?;
        input.parse::<Token![in]>()?;
        let begin: LitInt = input.parse()?;
        input.parse::<Token![..]>()?;
        // For test 7.
        let inclusive_range = if input.peek(Token![=]) {
            eprintln!("match inclusive range");
            input.parse::<Token![=]>()?;
            true
        } else {
            false
        };
        let end: LitInt = input.parse()?;
        let content;
        braced!(content in input);
        let content: proc_macro2::TokenStream = content.parse()?;
        
        // For test 7.
        let mut end_int = end.base10_parse::<isize>()?;
        if inclusive_range {
            end_int += 1;
        }

        Ok(SeqInput {
            ident,
            begin: begin.base10_parse()?,
            end: end_int,
            content,
        })
    }
}

impl SeqInput {
    fn expand(&self, ts: &proc_macro2::TokenStream, n: isize) -> proc_macro2::TokenStream {
        let tokens_iter = ts.clone().into_iter();
        let tokens = tokens_iter.collect::<Vec<_>>();
        let mut r = proc_macro2::TokenStream::new();

        let mut index = 0usize;
        while index < tokens.len() {
            let t0 = tokens.get(index).unwrap();
            let t1 = tokens.get(index + 1);
            let t2 = tokens.get(index + 2);
            // eprintln!("token: {}", t0);
            match &t0 {
                proc_macro2::TokenTree::Group(g) => {
                    let inner = self.expand(&g.stream(), n);
                    let mut inner_g = proc_macro2::Group::new(g.delimiter(), inner);
                    inner_g.set_span(g.clone().span());
                    r.extend(quote!{#inner_g});
                },
                proc_macro2::TokenTree::Ident(i) => {
                    if i == &self.ident {
                        let lit = proc_macro2::Literal::i64_unsuffixed(n as i64);
                        r.extend(quote!{#lit});
                    } else {
                        if let Some(ts) = self.match_paste_ident(n, t0, t1, t2) {
                            r.extend(quote!{#ts});
                            index += 2;
                        } else {
                            r.extend(quote!{#t0});
                        }   
                    }
                },
                _ => { r.extend(quote!{#t0}) },
            }
            index += 1;
        }
        r
    }

    fn match_paste_ident(&self, n: isize, t0: &TokenTree, t1: Option<&TokenTree>, t2: Option<&TokenTree>) -> Option<proc_macro2::TokenStream> {
        let t1 = t1?;
        let t2 = t2?;
        if &t1.to_string() == "~" {
            if let proc_macro2::TokenTree::Ident(i) = t2 {
                if i == &self.ident {
                    eprintln!("--- t1 & t2 matched");
                    let token_string = format!("{}{}", t0.to_string(), n);
                    let new_ident = proc_macro2::Ident::new(&token_string, t0.span());
                    let r = quote!{#new_ident};
                    return Some(r);
                }
            }
        }
        None
    }

    // For test 05.
    fn expand_repeat(&self) -> Option<proc_macro2::TokenStream> {
        let tb = TokenBuffer::new2(self.content.clone());
        let cursor = tb.begin();
        let mut matched = false;
        let output = self.expand_repeat_impl(cursor, &mut matched);
        // eprintln!("final match: {}", matched);
        // eprintln!("cursor output: {}", output);
        if matched {
            Some(output)
        } else {
            None
        }
    }

    fn expand_repeat_impl(&self, cursor: Cursor, matched: &mut bool) -> proc_macro2::TokenStream {
        let mut c = cursor;
        let mut output = proc_macro2::TokenStream::new();
        while !c.eof() {
            if let Some((punct, cursor)) = c.punct() {
                if punct.as_char() == '#' {
                    if let Some((c1, _, c2)) = cursor.group(Delimiter::Parenthesis) {
                        if let Some((punct, cursor)) = c2.punct() {
                            if punct.as_char() == '*' {
                                eprintln!("match repeated: {}", c1.token_stream());
                                for n in self.begin..self.end {
                                    output.extend(self.expand(&c1.token_stream(), n));
                                }
                                c = cursor;
                                *matched = true;
                                continue;
                            }
                        }
                    }
                }
            }

            if let Some((c1, _, c2)) = c.group(Delimiter::Brace) {
                // println!("Cursor: {{}}");
                let ts = self.expand_repeat_impl(c1, matched);
                output.extend(quote!{{#ts}});
                c = c2;
            } else if let Some((c1, _, c2)) = c.group(Delimiter::Bracket) {
                // println!("Cursor: []");
                let ts = self.expand_repeat_impl(c1, matched);
                output.extend(quote!{[#ts]});
                c = c2;
            } else if let Some((c1, _, c2)) = c.group(Delimiter::Parenthesis) {
                // println!("Cursor: ()");
                let ts = self.expand_repeat_impl(c1, matched);
                output.extend(quote!{(#ts)});
                c = c2;
            } else if let Some((punct, cursor)) = c.punct() {
                // println!("Cursor: {}", punct);
                output.extend(quote!{#punct});
                c = cursor;
            } else if let Some((ident, cursor)) = c.ident() {
                // println!("Cursor: {}", ident);
                output.extend(quote!{#ident});
                c = cursor;
            } else if let Some((literal, cursor)) = c.literal() {
                // println!("Cursor: {}", literal);
                output.extend(quote!{#literal});
                c = cursor;
            } else if let Some((lifetime, cursor)) = c.lifetime() {
                // println!("Cursor: {}", lifetime);
                output.extend(quote!{#lifetime});
                c = cursor;
            } else {
                panic!("Unavailable tokens");
            }
        }
        output
    }
}


#[proc_macro]
pub fn seq(input: TokenStream) -> TokenStream {
    let seq_input = parse_macro_input!(input as SeqInput);
    eprintln!(">>> [content]: {}", seq_input.content);
    let mut ret = proc_macro2::TokenStream::new();

    if let Some(output) = seq_input.expand_repeat() {
        eprintln!("Cursor output: {}", output);
        ret.extend(output);
    } else {
        eprintln!("Cursor output: None");
        for i in seq_input.begin..seq_input.end {
            ret.extend(seq_input.expand(&seq_input.content, i));
        }
    }

    eprintln!(">>> [output]: {}", ret);
    ret.into()
}
