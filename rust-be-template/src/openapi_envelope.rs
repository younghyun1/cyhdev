use utoipa::{
    Modify,
    openapi::{
        Content, OpenApi, Ref, RefOr,
        path::{Operation, PathItem},
        schema::{ObjectBuilder, Schema, SchemaType, Type},
    },
};

const META_SCHEMA: &str = "ApiResponseMeta";

/// Makes documented JSON success shapes match the runtime response envelope.
pub struct FrontendResponseEnvelope;

impl Modify for FrontendResponseEnvelope {
    fn modify(&self, openapi: &mut OpenApi) {
        add_meta_schema(openapi);
        for path_item in openapi.paths.paths.values_mut() {
            wrap_path_item(path_item);
        }
    }
}

fn add_meta_schema(openapi: &mut OpenApi) {
    let Some(components) = openapi.components.as_mut() else {
        return;
    };
    let schema = ObjectBuilder::new()
        .schema_type(Type::Object)
        .property(
            "time_to_process",
            ObjectBuilder::new().schema_type(Type::String),
        )
        .required("time_to_process")
        .property("timestamp", ObjectBuilder::new().schema_type(Type::String))
        .required("timestamp")
        .property(
            "metadata",
            ObjectBuilder::new().schema_type(SchemaType::AnyValue),
        )
        .required("metadata");
    components
        .schemas
        .insert(META_SCHEMA.to_owned(), schema.into());
}

fn wrap_path_item(path_item: &mut PathItem) {
    for operation in [
        &mut path_item.delete,
        &mut path_item.get,
        &mut path_item.head,
        &mut path_item.options,
        &mut path_item.patch,
        &mut path_item.post,
        &mut path_item.put,
        &mut path_item.trace,
    ] {
        if let Some(operation) = operation.as_mut() {
            wrap_operation(operation);
        }
    }
}

fn wrap_operation(operation: &mut Operation) {
    // This health probe intentionally returns its version document directly.
    if operation.operation_id.as_deref() == Some("healthcheck") {
        return;
    }

    for (status, response) in &mut operation.responses.responses {
        if !status.starts_with('2') {
            continue;
        }
        let RefOr::T(response) = response else {
            continue;
        };
        if let Some(content) = response.content.get_mut("application/json") {
            let data = content.schema.clone().unwrap_or_else(null_schema);
            if !is_envelope(&data) {
                content.schema = Some(envelope_schema(data));
            }
        } else if response.content.is_empty() {
            response.content.insert(
                "application/json".to_owned(),
                Content::new(Some(envelope_schema(null_schema()))),
            );
        }
    }
}

fn envelope_schema(data: RefOr<Schema>) -> RefOr<Schema> {
    ObjectBuilder::new()
        .schema_type(Type::Object)
        .property("success", ObjectBuilder::new().schema_type(Type::Boolean))
        .required("success")
        .property("data", data)
        .required("data")
        .property("meta", Ref::from_schema_name(META_SCHEMA))
        .required("meta")
        .into()
}

fn null_schema() -> RefOr<Schema> {
    ObjectBuilder::new().schema_type(Type::Null).into()
}

fn is_envelope(schema: &RefOr<Schema>) -> bool {
    let RefOr::T(Schema::Object(object)) = schema else {
        return false;
    };
    object.properties.contains_key("success")
        && object.properties.contains_key("data")
        && object.properties.contains_key("meta")
}
