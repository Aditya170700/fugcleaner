// Safety module - validation, hard-deny lists, canonicalization

pub const HARD_DENY_PATHS_UNIX: &[&str] = &[
    "/",
    "/System",
    "/Library",
    "/usr",
    "/bin",
    "/sbin",
    "/Applications",
    "/etc",
    "/var",
    "/dev",
    "/private",
];
