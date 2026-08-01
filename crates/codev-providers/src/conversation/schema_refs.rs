//! Reference-only utoipa shims for foreign (rmcp) types.
//!
//! utoipa 4 resolved schemas by NAME at document-assembly time, so a field in this
//! crate could `$ref` a schema that `codev-server` registers via its `derive_utoipa!`
//! bridge. utoipa 5 requires a concrete `ToSchema` bound instead, and `codev-server`
//! depends on this crate — so a direct reference would be circular.
//!
//! These shims break that cycle. Each one implements `PartialSchema::schema()` as a
//! bare `$ref` and carries NO body of its own, so:
//!
//!   * a field annotated `#[schema(value_type = TextContentRef)]` emits
//!     `$ref: #/components/schemas/TextContent`
//!   * the DEFINITION still comes from codev-server's bridge, which converts rmcp's
//!     schemars output — it is not duplicated or overwritten here
//!
//! Without these the fields fall back to `{"type":"object"}`, which is what made the
//! generated TypeScript client lose `.text` on message content (`useChatStream.ts`
//! saw `unknown`).
//!
//! INVARIANT: the string in each `Ref::new` must match a name registered by
//! `derive_utoipa!` in crates/codev-server/src/openapi.rs. If a name changes there,
//! change it here — a dangling `$ref` is a silently broken client, not a build error.

use std::borrow::Cow;
use utoipa::openapi::{schema::Schema, Ref, RefOr};
use utoipa::{PartialSchema, ToSchema};

macro_rules! schema_ref {
    ($shim:ident => $name:literal) => {
        #[doc = concat!("Reference-only shim emitting `$ref` to the `", $name, "` schema.")]
        pub struct $shim;

        impl PartialSchema for $shim {
            fn schema() -> RefOr<Schema> {
                RefOr::Ref(Ref::new(concat!("#/components/schemas/", $name)))
            }
        }

        impl ToSchema for $shim {
            fn name() -> Cow<'static, str> {
                // MUST differ from $name. utoipa keys BOTH the field $ref and the
                // components entry off name(), so returning $name here overwrites
                // codev-server's real definition with this shim's ref -> a
                // self-referential schema. Verified empirically, 2026-08-01.
                Cow::Borrowed(concat!($name, "Ref"))
            }
            // Deliberately does NOT push a schema: the definition is owned by
            // codev-server's derive_utoipa! bridge.
            fn schemas(_: &mut Vec<(String, RefOr<Schema>)>) {}
        }
    };
}

schema_ref!(RoleRef => "Role");
schema_ref!(TextContentRef => "TextContent");
schema_ref!(ImageContentRef => "ImageContent");
