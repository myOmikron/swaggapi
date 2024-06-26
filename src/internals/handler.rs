use std::ops::Deref;

use openapiv3::Responses;

use crate::handler_argument::HandlerArgumentFns;
use crate::internals::ptrset::PtrSet;
use crate::internals::HttpMethod;
use crate::internals::SchemaGenerator;
use crate::SwaggapiPageBuilder;

/// Meta information about a handler gathered by the [`#[handler]`](crate::handler) macro
#[derive(Copy, Clone, Debug)]
pub struct SwaggapiHandler {
    /// The http method the handler handles
    pub method: HttpMethod,

    /// The handler's path
    pub path: &'static str,

    /// `true` if `#[deprecated]` is present
    pub deprecated: bool,

    /// Set by macro if `#[doc = "..."]` (i.e. a doc comment) is present
    pub doc: &'static [&'static str],

    /// The handler's identifier
    pub ident: &'static str,

    /// Tags set through `#[operation(..., tags(...))]`
    pub tags: &'static [&'static str],

    /// The handler's return type's [`AsResponses::responses`](crate::as_responses::AsResponses::responses)
    pub responses: fn(&mut SchemaGenerator) -> Responses,

    /// The handler's arguments' [`HandlerArgument`](crate::handler_argument::HandlerArgument)'s methods
    pub handler_arguments: &'static [Option<HandlerArgumentFns>],

    /// The actual function stored in an actix specific format
    #[cfg(feature = "actix")]
    pub actix: fn() -> actix_web::Route,
    /// Placeholder to make the macro code cleaner
    #[cfg(not(feature = "actix"))]
    pub actix: (),

    /// The actual function stored in an axum specific format
    #[cfg(feature = "axum")]
    pub axum: fn() -> ::axum::routing::MethodRouter,
    /// Placeholder to make the macro code cleaner
    #[cfg(not(feature = "axum"))]
    pub axum: (),
}

#[cfg(not(feature = "actix"))]
#[macro_export]
#[doc(hidden)]
macro_rules! impl_Foo_actix {
    ($method:expr, $ident:ident) => {
        ()
    };
}
#[cfg(feature = "actix")]
#[macro_export]
#[doc(hidden)]
macro_rules! impl_Foo_actix {
    ($method:expr, $ident:ident) => {
        || {
            $crate::re_exports::actix_web::Route::new()
                .method($method.actix())
                .to($ident)
        }
    };
}

#[cfg(not(feature = "axum"))]
#[macro_export]
#[doc(hidden)]
macro_rules! impl_Foo_axum {
    ($method:expr, $ident:ident) => {
        ()
    };
}
#[cfg(feature = "axum")]
#[macro_export]
#[doc(hidden)]
macro_rules! impl_Foo_axum {
    ($method:expr, $ident:ident) => {
        || $crate::re_exports::axum::routing::MethodRouter::new().on($method.axum(), $ident)
    };
}

/// Representation of a [`SwaggapiHandler`] used inside [`ApiContext`](crate::ApiContext)
/// which allows modifications.
#[derive(Debug)]
pub struct ContextHandler {
    /// The original unmodified [`SwaggapiHandler`]
    pub original: SwaggapiHandler,

    /// The handler's modified path
    pub path: String,

    /// The handler's modified path
    pub tags: PtrSet<'static, str>,

    /// The pages the handler should be added to
    pub pages: PtrSet<'static, SwaggapiPageBuilder>,
}
impl ContextHandler {
    /// Constructs a new `ContextHandler`
    pub fn new(original: SwaggapiHandler) -> Self {
        Self {
            original,
            path: original.path.to_string(),
            tags: PtrSet::from_iter(original.tags.iter().copied()),
            pages: PtrSet::new(),
        }
    }
}
impl Deref for ContextHandler {
    type Target = SwaggapiHandler;

    fn deref(&self) -> &Self::Target {
        &self.original
    }
}
