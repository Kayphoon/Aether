pub(super) fn admin_provider_id_for_endpoints(request_path: &str) -> Option<String> {
    let raw = request_path.strip_prefix("/api/admin/endpoints/providers/")?;
    let raw = raw.strip_suffix("/endpoints")?;
    let normalized = raw.trim().trim_matches('/');
    if normalized.is_empty() {
        None
    } else {
        Some(normalized.to_string())
    }
}

pub(super) fn admin_endpoint_id(request_path: &str) -> Option<String> {
    let raw = request_path.strip_prefix("/api/admin/endpoints/")?;
    let normalized = raw.trim().trim_matches('/');
    if normalized.is_empty() || normalized.contains('/') {
        None
    } else {
        Some(normalized.to_string())
    }
}

pub(super) fn admin_default_body_rules_api_format(request_path: &str) -> Option<String> {
    let raw = request_path.strip_prefix("/api/admin/endpoints/defaults/")?;
    let raw = raw.strip_suffix("/body-rules")?;
    let normalized = raw.trim().trim_matches('/');
    if normalized.is_empty() {
        None
    } else {
        Some(normalized.to_string())
    }
}
