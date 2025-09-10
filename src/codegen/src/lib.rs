extern crate lazy_static;
use lazy_static::lazy_static;

use proc_macro::TokenStream;
use quote::quote;
use std::sync::atomic::{AtomicU32, Ordering};

lazy_static! {
    static ref NEXT_COMPONENT_ID: AtomicU32 = AtomicU32::new(1); // 0 is reserved
}

#[proc_macro_derive(Component)]
pub fn derive_component(item: TokenStream) -> TokenStream {
    let (ident, generics, id) = if let Ok(item) = syn::parse::<syn::ItemStruct>(item.clone()) {
        (
            item.ident,
            item.generics,
            NEXT_COMPONENT_ID.fetch_add(1, Ordering::Relaxed),
        )
    } else if let Ok(item) = syn::parse::<syn::ItemEnum>(item.clone()) {
        (
            item.ident,
            item.generics,
            NEXT_COMPONENT_ID.fetch_add(1, Ordering::Relaxed),
        )
    } else {
        panic!("Cannot use this macro here")
    };

    let ident_str = format!("{}", ident);
    quote! {
        impl #generics ecs::component::Component for #ident #generics {
            fn info(&self) -> ecs::component::ComponentInfo {
                ecs::component::ComponentInfo::new(ecs::component::ComponentId(#id), std::mem::size_of::<Self>(), #ident_str)
            }

            fn info_static() -> ecs::component::ComponentInfo {
                ecs::component::ComponentInfo::new(ecs::component::ComponentId(#id), std::mem::size_of::<Self>(), #ident_str)
            }
        }
    }
    .into()
}
