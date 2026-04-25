use proc_macro::TokenStream;
use quote::quote;
use syn::{
    parse::{Parse, ParseStream},
    parse_macro_input, Ident, LitStr, Result, Token,
};

enum RsxItem {
    Element(RsxElement),
    Text(LitStr),
}

struct RsxElement {
    tag: Ident,
    attributes: Vec<(Ident, LitStr)>,
    children: Vec<RsxItem>,
}

impl Parse for RsxElement {
    fn parse(input: ParseStream) -> Result<Self> {
        input.parse::<Token![<]>()?;
        let tag: Ident = input.parse()?;

        let mut attributes = Vec::new();
        while !input.peek(Token![>]) && !input.peek(Token![/]) {
            let attr: Ident = input.parse()?;
            input.parse::<Token![=]>()?;
            let value: LitStr = input.parse()?;
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
                .map(|(_, value)| quote!(#value.to_string()))
                .unwrap_or_else(|| quote!(String::new()));

            let attrs = el
                .attributes
                .iter()
                .filter(|(name, _)| name != "style")
                .map(|(name, value)| {
                    let key = name.to_string();
                    quote! {
                        node.set_attr(#key, #value.to_string());
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
    }
}

#[proc_macro]
pub fn rsx(input: TokenStream) -> TokenStream {
    let root = parse_macro_input!(input as RsxElement);
    generate_node(&RsxItem::Element(root)).into()
}
