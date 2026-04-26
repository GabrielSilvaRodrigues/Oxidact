use proc_macro::TokenStream;
use quote::quote;
use syn::{
    braced,
    parse::{Parse, ParseStream},
    parse_macro_input, Expr, Ident, LitStr, Result, Token,
};

enum RsxItem {
    Element(RsxElement),
    Text(LitStr),
    Expr(Expr),
}

struct RsxElement {
    tag: Ident,
    attributes: Vec<(Ident, AttrValue)>,
    children: Vec<RsxItem>,
}

enum AttrValue {
    Literal(LitStr),
    Expr(Expr),
}

impl AttrValue {
    fn to_tokens(&self) -> proc_macro2::TokenStream {
        match self {
            AttrValue::Literal(lit) => quote!(#lit),
            AttrValue::Expr(expr) => quote!((#expr)),
        }
    }
}

impl Parse for RsxElement {
    fn parse(input: ParseStream) -> Result<Self> {
        input.parse::<Token![<]>()?;
        let tag: Ident = input.parse()?;

        let mut attributes = Vec::new();
        while !input.peek(Token![>]) && !input.peek(Token![/]) {
            let attr: Ident = input.parse()?;
            input.parse::<Token![=]>()?;
            let value = if input.peek(LitStr) {
                AttrValue::Literal(input.parse()?)
            } else if input.peek(syn::token::Brace) {
                let content;
                braced!(content in input);
                AttrValue::Expr(content.parse()?)
            } else {
                return Err(syn::Error::new(
                    input.span(),
                    "atributo RSX deve ser string literal ou expressao entre chaves",
                ));
            };
            attributes.push((attr, value));
        }

        if input.peek(Token![/]) {
            input.parse::<Token![/]>()?;
            input.parse::<Token![>]>()?;
            return Ok(RsxElement {
                tag,
                attributes,
                children: Vec::new(),
            });
        }

        input.parse::<Token![>]>()?;

        let mut children = Vec::new();
        while !(input.peek(Token![<]) && input.peek2(Token![/])) {
            if input.peek(LitStr) {
                children.push(RsxItem::Text(input.parse()?));
            } else if input.peek(syn::token::Brace) {
                let content;
                braced!(content in input);
                children.push(RsxItem::Expr(content.parse()?));
            } else if input.peek(Token![<]) {
                children.push(RsxItem::Element(input.parse()?));
            } else {
                break;
            }
        }

        input.parse::<Token![<]>()?;
        input.parse::<Token![/]>()?;
        let close_tag: Ident = input.parse()?;
        input.parse::<Token![>]>()?;

        if close_tag != tag {
            return Err(syn::Error::new(close_tag.span(), "tag de fechamento diferente da tag de abertura"));
        }

        Ok(RsxElement {
            tag,
            attributes,
            children,
        })
    }
}

fn map_tag(tag: &str) -> proc_macro2::TokenStream {
    match tag {
        "Text" => quote!(oxidact_core::NodeType::Text),
        "TextInput" => quote!(oxidact_core::NodeType::TextInput),
        "Pressable" => quote!(oxidact_core::NodeType::Pressable),
        "SafeAreaView" => quote!(oxidact_core::NodeType::SafeAreaView),
        "NavigationContainer" => quote!(oxidact_core::NodeType::NavigationContainer),
        "StackNavigator" => quote!(oxidact_core::NodeType::StackNavigator),
        "StackScreen" => quote!(oxidact_core::NodeType::StackScreen),
        _ => quote!(oxidact_core::NodeType::View),
    }
}

fn generate_node(item: &RsxItem) -> proc_macro2::TokenStream {
    match item {
        RsxItem::Element(el) => {
            let tag_name = el.tag.to_string();
            let node_type = map_tag(&tag_name);
            let style = el
                .attributes
                .iter()
                .find(|(name, _)| name == "style")
                .map(|(_, value)| {
                    let v = value.to_tokens();
                    quote!((#v).to_string())
                })
                .unwrap_or_else(|| quote!(String::new()));

            let attrs = el
                .attributes
                .iter()
                .filter(|(name, _)| name != "style")
                .map(|(name, value)| {
                    let key = name.to_string();
                    let v = value.to_tokens();
                    quote! {
                        node.set_attr(#key, (#v).to_string());
                    }
                });

            let children = el.children.iter().map(generate_node);

            quote! {
                {
                    let mut node = oxidact_core::VNode::new(#node_type);
                    node.style_raw = #style;
                    #(#attrs)*
                    node.children = vec![#(#children),*];
                    node
                }
            }
        }
        RsxItem::Text(text) => {
            quote! {
                {
                    let mut node = oxidact_core::VNode::new(oxidact_core::NodeType::Text);
                    node.text_content = Some(#text.to_string());
                    node
                }
            }
        }
        RsxItem::Expr(expr) => {
            quote! {
                (#expr)
            }
        }
    }
}

#[proc_macro]
pub fn rsx(input: TokenStream) -> TokenStream {
    let root = parse_macro_input!(input as RsxElement);
    generate_node(&RsxItem::Element(root)).into()
}
