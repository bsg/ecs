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
    let vis = item_struct.vis;
    let ident = item_struct.ident;
    let generics = item_struct.generics;
    let id = NEXT_COMPONENT_ID.fetch_add(1, Ordering::Relaxed);

    quote! {
        #vis impl #generics ecs::component::Component for #ident #generics {
            fn info(&self) -> ecs::component::ComponentInfo {
                ecs::component::ComponentInfo::new(ecs::component::ComponentId(#id), std::mem::size_of::<Self>())
            }

            fn info_static() -> ecs::component::ComponentInfo {
                ecs::component::ComponentInfo::new(ecs::component::ComponentId(#id), std::mem::size_of::<Self>())
            }
        }

        #vis impl<'access> From<ecs::Access<'access, #ident>> for &'access #ident {
            fn from(access: ecs::Access<'access, #ident>) -> Self {
                unsafe {access.store.read::<#ident>(access.index)}
            }
        }

        #vis impl<'access> From<ecs::Access<'access, #ident>> for &'access mut #ident {
            fn from(access: ecs::Access<'access, #ident>) -> Self {
                unsafe {access.store.read_mut::<#ident>(access.index)}
            }
        }

        #vis impl<'access> From<ecs::Access<'access, #ident>> for Option<&'access #ident> {
            fn from(access: ecs::Access<'access, #ident>) -> Self {
                unsafe {access.store.try_read::<#ident>(access.index)}
            }
        }

        #vis impl<'access> From<ecs::Access<'access, #ident>> for Option<&'access mut #ident> {
            fn from(access: ecs::Access<'access, #ident>) -> Self {
                unsafe {access.store.try_read_mut::<#ident>(access.index)}
            }
        }
    }
    .into()
}
