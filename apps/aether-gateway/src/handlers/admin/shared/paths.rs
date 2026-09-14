pub(crate) fn is_admin_gemini_files_mappings_root(request_path: &str) -> bool {
    matches!(
        request_path,
        "/api/admin/gemini-files/mappings" | "/api/admin/gemini-files/mappings/"
    )
}

pub(crate) fn is_admin_gemini_files_stats_root(request_path: &str) -> bool {
    matches!(
        request_path,
        "/api/admin/gemini-files/stats" | "/api/admin/gemini-files/stats/"
    )
}

pub(crate) fn is_admin_gemini_files_capable_keys_root(request_path: &str) -> bool {
    matches!(
        request_path,
        "/api/admin/gemini-files/capable-keys" | "/api/admin/gemini-files/capable-keys/"
    )
}

pub(crate) fn is_admin_gemini_files_upload_root(request_path: &str) -> bool {
    matches!(
        request_path,
        "/api/admin/gemini-files/upload" | "/api/admin/gemini-files/upload/"
    )
}

pub(crate) fn admin_gemini_file_mapping_id_from_path(request_path: &str) -> Option<String> {
    let raw = request_path.strip_prefix("/api/admin/gemini-files/mappings/")?;
    let normalized = raw.trim().trim_matches('/');
    if normalized.is_empty() || normalized.contains('/') {
        None
    } else {
        Some(normalized.to_string())
    }
}
