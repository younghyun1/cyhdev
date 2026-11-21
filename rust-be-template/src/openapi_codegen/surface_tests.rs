//! The browser contract list names every operation the admin panels call.

use super::surface::{FRONTEND_OPERATIONS, FrontendOperation};

#[test]
fn selects_every_admin_operations_panel_contract() {
    let expected = [
        FrontendOperation {
            client_group: "account",
            method: "GET",
            path: "/api/admin/account-retention-notifications",
        },
        FrontendOperation {
            client_group: "account",
            method: "POST",
            path: "/api/admin/account-retention-notifications/{notification_id}/retry",
        },
        FrontendOperation {
            client_group: "account",
            method: "POST",
            path: "/api/admin/users/{user_id}/hard-purge",
        },
        FrontendOperation {
            client_group: "account",
            method: "GET",
            path: "/api/admin/media-cleanup/unresolved",
        },
        FrontendOperation {
            client_group: "account",
            method: "POST",
            path: "/api/admin/media-cleanup/{cleanup_id}/resolve",
        },
        FrontendOperation {
            client_group: "i18n",
            method: "POST",
            path: "/api/admin/sync-i18n-cache",
        },
    ];

    for operation in expected {
        assert!(
            FRONTEND_OPERATIONS.contains(&operation),
            "missing generated frontend operation: {} {}",
            operation.method,
            operation.path,
        );
    }
}
