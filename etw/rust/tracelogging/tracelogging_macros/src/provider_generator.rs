use proc_macro::*;

use crate::provider_info::ProviderInfo;
use crate::strings::*;
use crate::tree::Tree;

pub struct ProviderGenerator {
    prov_tree: Tree,
    tree1: Tree,
    tree2: Tree,
    tree3: Tree,
}

impl ProviderGenerator {
    pub fn new(span: Span) -> Self {
        return Self {
            prov_tree: Tree::new(span),
            tree1: Tree::new(span),
            tree2: Tree::new(span),
            tree3: Tree::new(span),
        };
    }

    pub fn generate(&mut self, provider: ProviderInfo) -> TokenStream {
        // Reserve space for size.
        let mut meta = Vec::<u8>::new();
        meta.push(0);
        meta.push(0);

        // Provider name + NUL
        meta.extend_from_slice(provider.name.as_bytes());
        meta.push(0);

        if let Some(ref group_id) = provider.group_id {
            // Provider group id
            meta.push(19); // size is 19: sizeof(size) + sizeof(type) + sizeof(guid) = 2 + 1 + 16
            meta.push(0);
            meta.push(1); // EtwProviderTraitTypeGroup
            meta.extend_from_slice(&group_id.to_bytes_le());
        }

        meta[0] = meta.len() as u8;
        meta[1] = (meta.len() >> 8) as u8;

        let id_fields = provider.id.to_fields();

        let prov_tokens = self
            .prov_tree
            // static PROVIDER: ::tracelogging::Provider = unsafe { ... }
            .add_ident("static")
            .add(provider.symbol)
            .add_punct(":")
            .add_path(PROVIDER_PATH)
            .add_punct("=")
            .add_ident("unsafe")
            .add_group_curly(
                self.tree1
                    .add_path_call(
                        // ::tracelogging::_internal::provider_new(
                        PROVIDER_NEW_PATH,
                        self.tree2
                            // b"EncodedProviderMetadata...",
                            .add(Literal::byte_string(&meta))
                            .add_punct(",")
                            // &::tracelogging::Guid::from_fields(d0, d1, d2, *b"d3...")
                            .add_punct("&")
                            .add_path_call(
                                GUID_FROM_FIELDS_PATH,
                                self.tree3
                                    .add(Literal::u32_unsuffixed(id_fields.0))
                                    .add_punct(",")
                                    .add(Literal::u16_unsuffixed(id_fields.1))
                                    .add_punct(",")
                                    .add(Literal::u16_unsuffixed(id_fields.2))
                                    .add_punct(",")
                                    .add_punct("*")
                                    .add(Literal::byte_string(&id_fields.3))
                                    .drain(),
                            )
                            .drain(),
                    )
                    .drain(),
            )
            .add_punct(";")
            .drain()
            .collect();

        if provider.debug {
            println!("{}", prov_tokens);
        }

        return prov_tokens;
    }
}
