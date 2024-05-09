extern crate lazy_static;
use lazy_static::lazy_static;

use proc_macro::TokenStream;
use quote::quote;
use std::sync::atomic::{AtomicU32, Ordering};

lazy_static! {
    static ref NEXT_COMPONENT_ID: AtomicU32 = AtomicU32::new(0);
}

#[proc_macro_derive(Component)]
pub fn derive_component(item: TokenStream) -> TokenStream {
    let item_struct =
        syn::parse::<syn::ItemStruct>(item.clone()).expect("Cannot use this macro here");
    let struct_vis = item_struct.vis;
    let struct_ident = item_struct.ident;
    let struct_generics = item_struct.generics;
    let id = NEXT_COMPONENT_ID.fetch_add(1, Ordering::Relaxed);

    quote! {
        #struct_vis impl #struct_generics ecs::component::Component for #struct_ident #struct_generics {
            fn info(&self) -> ecs::component::ComponentInfo {
                ecs::component::ComponentInfo::new(ecs::component::ComponentId(#id), std::mem::size_of::<Self>())
            }

            fn info_static() -> ecs::component::ComponentInfo {
                ecs::component::ComponentInfo::new(ecs::component::ComponentId(#id), std::mem::size_of::<Self>())
            }
        }
    }
    .into()
}
