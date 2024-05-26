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
    let item_struct =
        syn::parse::<syn::ItemStruct>(item.clone()).expect("Cannot use this macro here");
    let ident = item_struct.ident;
    let generics = item_struct.generics;
    let id = NEXT_COMPONENT_ID.fetch_add(1, Ordering::Relaxed);

    quote! {
        impl #generics ecs::component::Component for #ident #generics {
            fn info(&self) -> ecs::component::ComponentInfo {
                ecs::component::ComponentInfo::new(ecs::component::ComponentId(#id), std::mem::size_of::<Self>())
            }

            fn info_static() -> ecs::component::ComponentInfo {
                ecs::component::ComponentInfo::new(ecs::component::ComponentId(#id), std::mem::size_of::<Self>())
            }
        }

        impl #generics Clone for #ident #generics {
            fn clone(&self) -> Self {
                self.clone()
            }
        }

        impl #generics Copy for #ident #generics {}
    }
    .into()
}

#[proc_macro_derive(Resource)]
pub fn derive_resource(item: TokenStream) -> TokenStream {
    let item_struct =
        syn::parse::<syn::ItemStruct>(item.clone()).expect("Cannot use this macro here");
    let ident = item_struct.ident;
    let generics = item_struct.generics;

    quote! {
        impl #generics ecs::Resource for #ident #generics {
            fn as_any(&self) -> &dyn core::any::Any {
                self
            }

            fn as_mut_any(&mut self) -> &mut dyn core::any::Any {
                self
            }
        }

        impl #generics ecs::component::Component for #ident #generics {
            fn info(&self) -> ecs::component::ComponentInfo {
                ecs::component::ComponentInfo::new(ecs::component::ComponentId(0), 0) // ignored
            }

            fn info_static() -> ecs::component::ComponentInfo {
                ecs::component::ComponentInfo::new(ecs::component::ComponentId(0), 0) // ignored
            }
        }
    }
    .into()
}
