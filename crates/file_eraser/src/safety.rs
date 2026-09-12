use crate::errors::EraserResult;
use crate::model::{SafetyClassification, TargetKind};
use crate::platform::PlatformProvider;
use std::path::{Component, Path, PathBuf};

/// Check if `candidate` is a descendant of or identical to `base`.
/// Component-aware and Windows case-insensitive.
///
/// Example:
/// "D:\Windows\System32" in "D:\Windows" => true
/// "D:\WindowsBackup\file.txt" in "D:\Windows" => false
pub fn is_descendant_or_equal<P1: AsRef<Path>, P2: AsRef<Path>>(candidate: P1, base: P2) -> bool {
    let c_comps: Vec<Component> = candidate.as_ref().components().collect();
    let b_comps: Vec<Component> = base.as_ref().components().collect();

    if c_comps.len() < b_comps.len() {
        return false;
    }

    for (c, b) in c_comps.iter().zip(b_comps.iter()) {
        if !component_equal(c, b) {
            return false;
        }
    }
    true
}

fn component_equal(a: &Component, b: &Component) -> bool {
    match (a, b) {
        (Component::Prefix(p1), Component::Prefix(p2)) => {
            p1.as_os_str().to_string_lossy().to_uppercase()
                == p2.as_os_str().to_string_lossy().to_uppercase()
        }
        (Component::RootDir, Component::RootDir) => true,
        (Component::CurDir, Component::CurDir) => true,
        (Component::ParentDir, Component::ParentDir) => true,
        (Component::Normal(n1), Component::Normal(n2)) => {
            n1.to_string_lossy().to_uppercase() == n2.to_string_lossy().to_uppercase()
        }
        _ => false,
    }
}

pub struct SafetyEngine<P: PlatformProvider> {
    platform: P,
    cached_protected_roots: Vec<PathBuf>,
}

impl<P: PlatformProvider> SafetyEngine<P> {
    pub fn new(platform: P) -> EraserResult<Self> {
        let roots = platform.get_protected_system_paths()?;
        let mut normalized_roots = Vec::new();
        for r in roots {
            let norm = platform.normalize_path(&r).unwrap_or(r);
            normalized_roots.push(norm);
        }
        Ok(Self {
            platform,
            cached_protected_roots: normalized_roots,
        })
    }

    /// Evaluates target safety completely read-only.
    /// Does not guess active system files without authoritative backing.
    pub fn classify_target(&self, path: &Path, kind: &TargetKind) -> SafetyClassification {
        let norm_target = self
            .platform
            .normalize_path(path)
            .unwrap_or_else(|_| path.to_path_buf());

        // Device path protection
        let path_str = norm_target.to_string_lossy().to_uppercase();
        if path_str.starts_with(r"\\.\PHYSICALDRIVE")
            || path_str.starts_with(r"\\?\VOLUME{") && path_str.ends_with(r"}\")
        {
            return SafetyClassification::BlockedDevicePath {
                reason: "Raw storage device paths cannot be targeted for file erasure".into(),
            };
        }

        // Reparse point safety: do not cross
        if kind.is_reparse_boundary() {
            return SafetyClassification::BlockedReparsePoint {
                reason: "Target is a reparse point (symlink/junction/boundary); recursion blocked to prevent escaping target scope".into(),
            };
        }

        // Inaccessible
        if let TargetKind::Inaccessible { reason } = kind {
            return SafetyClassification::Inaccessible {
                reason: reason.clone(),
            };
        }

        // Protected system directories: Component-aware check
        for protected in &self.cached_protected_roots {
            if is_descendant_or_equal(&norm_target, protected) {
                return SafetyClassification::ProtectedSystemPath {
                    reason: format!(
                        "Path resides within protected system directory: {:?}",
                        protected
                    ),
                };
            }
        }

        SafetyClassification::SafeToAnalyze
    }
}
