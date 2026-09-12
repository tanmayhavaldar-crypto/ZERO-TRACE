use crate::errors::EraserResult;
use crate::filesystem_probe::FilesystemProber;
use crate::model::*;
use crate::platform::PlatformProvider;
use crate::safety::is_descendant_or_equal;
use crate::traversal::DirectoryCrawler;
use std::collections::HashSet;
use std::ffi::OsString;
use std::path::{Component, Path, PathBuf};

/// Generates a platform-aware comparison key for path deduplication and containment checks.
/// On Windows, drive letters, prefixes, and normal components are normalized to uppercase.
/// On non-Windows platforms, components retain their original case.
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
struct PathDeduplicationKey {
    components: Vec<ComponentKey>,
}

#[derive(Debug, Clone, PartialEq, Eq, Hash)]
enum ComponentKey {
    Prefix(String),
    RootDir,
    CurDir,
    ParentDir,
    Normal(OsString),
}

impl PathDeduplicationKey {
    fn from_path(path: &Path) -> Self {
        let components = path
            .components()
            .map(|c| match c {
                Component::Prefix(p) => {
                    #[cfg(windows)]
                    {
                        ComponentKey::Prefix(p.as_os_str().to_string_lossy().to_uppercase())
                    }
                    #[cfg(not(windows))]
                    {
                        ComponentKey::Prefix(p.as_os_str().to_string_lossy().into_owned())
                    }
                }
                Component::RootDir => ComponentKey::RootDir,
                Component::CurDir => ComponentKey::CurDir,
                Component::ParentDir => ComponentKey::ParentDir,
                Component::Normal(n) => {
                    #[cfg(windows)]
                    {
                        ComponentKey::Normal(OsString::from(n.to_string_lossy().to_uppercase()))
                    }
                    #[cfg(not(windows))]
                    {
                        ComponentKey::Normal(n.to_os_string())
                    }
                }
            })
            .collect();

        Self { components }
    }
}

pub struct TargetResolver<P: PlatformProvider> {
    platform: P,
    crawler: DirectoryCrawler<P>,
    prober: FilesystemProber<P>,
}

impl<P: PlatformProvider + Clone> TargetResolver<P> {
    pub fn new(platform: P) -> EraserResult<Self> {
        let crawler = DirectoryCrawler::new(platform.clone())?;
        let prober = FilesystemProber::new(platform.clone())?;
        Ok(Self {
            platform,
            crawler,
            prober,
        })
    }

    /// Resolves and deduplicates a batch of arbitrary input targets.
    ///
    /// Deduplication rules:
    /// 1. If path A is identical to path B (case-insensitive on Windows), deduplicate
    ///    using a platform-aware deduplication key while preserving the actual path.
    /// 2. If path A is a descendant of directory B, and both are in the batch, directory B's
    ///    recursive crawl naturally discovers A. Redundant top-level crawls are suppressed.
    pub fn resolve_targets(
        &self,
        raw_paths: &[PathBuf],
    ) -> EraserResult<(Vec<TargetProbeData>, ResolutionStatistics)> {
        let mut stats = ResolutionStatistics {
            total_input_targets: raw_paths.len(),
            ..Default::default()
        };

        let mut normalized_targets: Vec<PathBuf> = Vec::new();
        let mut seen_keys = HashSet::<PathDeduplicationKey>::new();

        for p in raw_paths {
            let norm = self
                .platform
                .normalize_path(p)
                .unwrap_or_else(|_| p.clone());

            let key = PathDeduplicationKey::from_path(&norm);
            if seen_keys.insert(key) {
                normalized_targets.push(norm);
            }
        }

        stats.unique_targets = normalized_targets.len();

        // Detect directory containment overlap:
        // If target A is inside directory target B, mark A as subsumed to avoid duplicate scanning.
        let mut filtered_roots: Vec<PathBuf> = Vec::new();
        for candidate in &normalized_targets {
            let mut subsumed = false;
            for other in &normalized_targets {
                if !is_path_equal(candidate, other)
                    && self.is_directory_candidate(other)
                    && is_descendant_or_equal(candidate, other)
                {
                    subsumed = true;
                    break;
                }
            }
            if !subsumed {
                filtered_roots.push(candidate.clone());
            }
        }

        let mut all_results = Vec::new();
        for target in filtered_roots {
            let probe = self.prober.probe_target(&target)?;
            if probe.kind.is_traversable_directory()
                && matches!(probe.safety, SafetyClassification::SafeToAnalyze)
            {
                let mut dir_results = self.crawler.crawl(&target, &mut stats)?;
                all_results.append(&mut dir_results);
            } else {
                self.tally_single(&probe, &mut stats);
                all_results.push(probe);
            }
        }

        Ok((all_results, stats))
    }

    fn is_directory_candidate(&self, path: &Path) -> bool {
        match self.platform.query_target_kind(path) {
            Ok(TargetKind::Directory) => true,
            _ => false,
        }
    }

    fn tally_single(&self, probe: &TargetProbeData, stats: &mut ResolutionStatistics) {
        match &probe.kind {
            TargetKind::RegularFile => stats.regular_files_discovered += 1,
            TargetKind::Directory => stats.directories_discovered += 1,
            k if k.is_reparse_boundary() => stats.reparse_boundaries_encountered += 1,
            TargetKind::Inaccessible { .. } => stats.inaccessible_targets_encountered += 1,
            _ => {}
        }
        if matches!(probe.safety, SafetyClassification::ProtectedSystemPath { .. }) {
            stats.protected_targets_encountered += 1;
        }
    }
}

fn is_path_equal(a: &Path, b: &Path) -> bool {
    PathDeduplicationKey::from_path(a) == PathDeduplicationKey::from_path(b)
}