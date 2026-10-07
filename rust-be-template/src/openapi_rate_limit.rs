//! Documents the admission response shared by every non-Minecraft HTTP operation.

use utoipa::{
    Modify,
    openapi::{
        Content, OpenApi, Ref, RefOr,
        header::HeaderBuilder,
        response::ResponseBuilder,
        schema::{ObjectBuilder, Type},
    },
};

use crate::routers::middleware::request_rate_limit::minecraft_exempt;

/// Keeps cross-cutting HTTP admission in the same contract as feature responses.
pub struct GlobalRequestAdmission;

impl Modify for GlobalRequestAdmission {
    fn modify(&self, openapi: &mut OpenApi) {
        for (path, item) in &mut openapi.paths.paths {
            if minecraft_exempt(path) {
                continue;
            }
            for operation in [
                &mut item.delete,
                &mut item.get,
                &mut item.head,
                &mut item.options,
                &mut item.patch,
                &mut item.post,
                &mut item.put,
                &mut item.trace,
            ]
            .into_iter()
            .flatten()
            {
                let response = operation.responses.responses.entry("429".to_owned())
                    .or_insert_with(|| ResponseBuilder::new()
                        .description("The shared client request budget or tracking capacity is exhausted")
                        .content("application/json", Content::new(Some(Ref::from_schema_name("CodeErrorResp"))))
                        .into());
                if let RefOr::T(response) = response {
                    response
                        .headers
                        .entry("Retry-After".to_owned())
                        .or_insert_with(|| {
                            HeaderBuilder::new()
                                .description(Some("Seconds to wait before retrying"))
                                .schema(Some(ObjectBuilder::new().schema_type(Type::Integer)))
                                .build()
                                .into()
                        });
                }
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::docs::ApiDoc;
    use utoipa::OpenApi as _;

    #[test]
    fn documents_shared_rejection_without_changing_minecraft_operations() {
        let doc = ApiDoc::openapi();
        for (path, item) in &doc.paths.paths {
            for operation in [
                &item.delete,
                &item.get,
                &item.head,
                &item.options,
                &item.patch,
                &item.post,
                &item.put,
                &item.trace,
            ]
            .into_iter()
            .flatten()
            {
                if !minecraft_exempt(path) {
                    assert!(
                        matches!(operation.responses.responses.get("429"),
                        Some(RefOr::T(response)) if response.headers.contains_key("Retry-After")),
                        "{path}"
                    );
                }
            }
        }
        let action = &doc.paths.paths["/api/admin/minecraft"].get;
        assert!(
            action
                .as_ref()
                .is_some_and(|op| !op.responses.responses.contains_key("429"))
        );
    }
}
