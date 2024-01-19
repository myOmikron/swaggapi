// #![warn(missing_docs)]
#![warn(clippy::todo)]

pub mod as_responses;
mod convert;
mod description;
pub mod handler;
pub mod handler_argument;
mod method;

use std::collections::BTreeMap;
use std::mem;
use std::sync::{Arc, Mutex};

pub use convert::convert_schema;
use indexmap::IndexMap;
pub use swaggapi_macro::*;

pub use self::description::OperationDescription;
pub use self::method::Method;

/// Reexports for macros and implementors
pub mod re_exports {
    #[cfg(feature = "actix")]
    pub use actix_web;
    #[cfg(feature = "axum")]
    pub use axum;
    pub use {indexmap, openapiv3, schemars};
}

use openapiv3::{Components, Info, OpenAPI, PathItem, Paths, ReferenceOr};
use schemars::gen::{SchemaGenerator, SchemaSettings};
use schemars::schema::Schema;

use crate::handler::Handler;

#[derive(SwaggapiPage)]
pub struct PageOfEverything;

pub trait SwaggapiPage {
    fn builder() -> &'static SwaggapiPageBuilder;
}

pub struct SwaggapiPageBuilder {
    title: &'static str,
    version: &'static str,
    state: Mutex<Option<SwaggapiPageBuilderState>>,
}

#[derive(Default)]
struct SwaggapiPageBuilderState {
    paths: Paths,

    /// The linkable schemas generated by [`SchemaGenerator`].
    ///
    /// (See [`SwaggapiPageBuilderState::generate_schema`] for more info)
    schemas: BTreeMap<String, Schema>,

    /// Cache for the result of [`SwaggapiPageBuilder::build`]
    last_build: Option<Arc<OpenAPI>>,
}

impl SwaggapiPageBuilder {
    pub const fn new() -> Self {
        Self {
            title: "",
            version: "",
            state: Mutex::new(None),
        }
    }

    pub fn add_handler(&self, handler: &impl Handler) {
        let mut guard = self.state.lock().unwrap();
        let state = guard.get_or_insert_with(Default::default);
        state.last_build = None;

        let operation = state
            .generate_schema(|gen| handler.description(gen))
            .build();
        let ReferenceOr::Item(path) = state
            .paths
            .paths
            .entry(format!("{}/{}", handler.ctx_path(), handler.path()))
            .or_insert_with(|| ReferenceOr::Item(PathItem::default()))
        else {
            unreachable!("We only ever insert ReferenceOr::Item. See above")
        };
        let operation_mut = match handler.method() {
            Method::Get => &mut path.get,
            Method::Post => &mut path.post,
            Method::Put => &mut path.put,
            Method::Delete => &mut path.delete,
            Method::Head => &mut path.head,
            Method::Options => &mut path.options,
            Method::Patch => &mut path.patch,
            Method::Trace => &mut path.trace,
        };
        *operation_mut = Some(operation);
    }

    pub fn build(&self) -> Arc<OpenAPI> {
        let mut guard = self.state.lock().unwrap();
        let state = guard.get_or_insert_with(Default::default);

        if let Some(open_api) = state.last_build.clone() {
            return open_api;
        }

        let open_api = Arc::new(OpenAPI {
            openapi: "3.0".to_string(),
            info: Info {
                title: self.title.to_string(),
                description: None,
                terms_of_service: None,
                contact: None,
                license: None,
                version: self.version.to_string(),
                extensions: IndexMap::new(),
            },
            servers: vec![],
            paths: state.paths.clone(),
            components: Some(Components {
                schemas: state
                    .schemas
                    .iter()
                    .map(|(key, schema)| (key.clone(), convert_schema(schema.clone())))
                    .collect(),
                ..Default::default()
            }),
            security: None,
            tags: vec![],
            external_docs: None,
            extensions: IndexMap::new(),
        });

        state.last_build = Some(open_api.clone());
        open_api
    }
}

impl SwaggapiPageBuilderState {
    /// Generate a schema, writing sub schemas to the state.
    ///
    /// [`SchemaGenerator`] is not [`Send`], so we can't just put it into our [`SwaggapiPageBuilderState`]
    /// which is "shared" across thread.
    /// So as awkward workaround, we only store [`SchemaGenerator`]'s relevant state, get a new generator for each schema
    /// and temporarily swap the relevant state with us.
    fn generate_schema<T>(&mut self, func: impl FnOnce(&mut SchemaGenerator) -> T) -> T {
        // Create SchemaGenerator and give him our schemas
        let mut settings = SchemaSettings::openapi3();
        settings.visitors = Vec::new();
        let mut gen = SchemaGenerator::new(settings);
        let definitions_mut = gen.definitions_mut();
        *definitions_mut = mem::take(&mut self.schemas);

        // Generate the new schema
        let schema = func(&mut gen);

        // Take our (potentially modified) schemas back
        self.schemas = gen.take_definitions();

        schema
    }
}
