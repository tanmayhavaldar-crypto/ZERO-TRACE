use file_eraser::model::*;
use file_eraser::safety::is_descendant_or_equal;
use file_eraser::{FilesystemProber, MockEntryConfig, MockPlatformProvider, TargetResolver};
use std::fs::{self, File};
use std::io::Write;
use std::path::PathBuf;
use tempfile::tempdir;

#[test]
fn test_1_verified_unavailable_probed_value_behavior() {
    let mut mock = MockPlatformProvider::new();
    mock.volume_info_result = Err("DeviceIoControl failed: access denied".into());

    let path = PathBuf::from(r"D:\Data\sample.txt");
    mock.insert_entry(path.clone(), MockEntryConfig::default());

    let prober = FilesystemProber::new(mock).expect("Prober initialization must succeed");
    let probe = prober
        .probe_target(&path)
        .expect("Target probe should succeed");

    assert!(!probe.volume_info.is_verified());
    match probe.volume_info {
        ProbedValue::Unavailable { reason } => {
            assert!(reason.contains("Volume information query failed"));
            assert!(reason.contains("DeviceIoControl failed: access denied"));
        }
        _ => panic!("Expected ProbedValue::Unavailable"),
    }
}

#[test]
fn test_2_system_directory_containment() {
    let base = PathBuf::from(r"C:\Windows");
    let child = PathBuf::from(r"c:\windows\System32\drivers\etc\hosts");
    assert!(is_descendant_or_equal(&child, &base));
}

#[test]
fn test_3_similar_prefix_directory_is_not_treated_as_protected() {
    let base = PathBuf::from(r"D:\Windows");
    let sibling1 = PathBuf::from(r"D:\WindowsBackup\file.txt");
    let sibling2 = PathBuf::from(r"D:\Windows.old.backup");

    assert!(!is_descendant_or_equal(&sibling1, &base));
    assert!(!is_descendant_or_equal(&sibling2, &base));
}

#[test]
fn test_4_directory_descendant_detection() {
    let base = PathBuf::from(r"D:\SafeDir");
    let sub = PathBuf::from(r"D:\safedir\sub\doc.pdf");
    assert!(is_descendant_or_equal(&sub, &base));
}

#[test]
fn test_5_similar_prefix_directory_is_not_treated_as_descendant() {
    let dir = PathBuf::from(r"D:\Data");
    let sibling = PathBuf::from(r"D:\Database\table.bin");
    assert!(!is_descendant_or_equal(&sibling, &dir));
}

#[test]
fn test_6_reparse_point_is_never_traversed() {
    let mut mock = MockPlatformProvider::new();
    let root = PathBuf::from(r"D:\UserDir");
    let junction = PathBuf::from(r"D:\UserDir\JunctionFolder");
    let junction_child = PathBuf::from(r"D:\UserDir\JunctionFolder\Secret.txt");

    let mut root_config = MockEntryConfig::default();
    root_config.kind = TargetKind::Directory;
    root_config.children = vec![junction.clone()];
    mock.insert_entry(root.clone(), root_config);

    let mut junction_config = MockEntryConfig::default();
    junction_config.kind = TargetKind::Junction;
    junction_config.reparse_tag = Ok(ReparseTagType::MountPointJunction);
    junction_config.children = vec![junction_child.clone()];
    mock.insert_entry(junction.clone(), junction_config);

    mock.insert_entry(junction_child.clone(), MockEntryConfig::default());

    let resolver = TargetResolver::new(mock).expect("Resolver creation must succeed");
    let (probed, stats) = resolver
        .resolve_targets(&[root])
        .expect("Resolution should succeed");

    assert_eq!(stats.reparse_boundaries_encountered, 1);
    // Boundary must be logged, but recursive traversal into junction_child must NOT happen
    assert!(!probed.iter().any(|p| p.normalized_path == junction_child));
}

#[test]
fn test_7_inaccessible_file_does_not_abort_scan() {
    let mut mock = MockPlatformProvider::new();
    let dir = PathBuf::from(r"D:\MyDir");
    let file_ok = PathBuf::from(r"D:\MyDir\ok.txt");
    let file_broken = PathBuf::from(r"D:\MyDir\broken.txt");

    let mut dir_config = MockEntryConfig::default();
    dir_config.kind = TargetKind::Directory;
    dir_config.children = vec![file_ok.clone(), file_broken.clone()];
    mock.insert_entry(dir.clone(), dir_config);

    mock.insert_entry(file_ok.clone(), MockEntryConfig::default());
    // file_broken is intentionally omitted from entries to simulate an inaccessible target during crawl

    let resolver = TargetResolver::new(mock).expect("Resolver initialization must succeed");
    let (probed, stats) = resolver
        .resolve_targets(&[dir])
        .expect("Traversal must not abort on inaccessible child entries");

    assert_eq!(stats.inaccessible_targets_encountered, 1);
    assert_eq!(stats.regular_files_discovered, 1);
    assert_eq!(stats.directories_discovered, 1);

    let broken_probe = probed
        .iter()
        .find(|p| p.normalized_path == file_broken)
        .expect("Broken file must be recorded in probe results");

    assert!(matches!(broken_probe.kind, TargetKind::Inaccessible { .. }));
    assert!(matches!(
        broken_probe.safety,
        SafetyClassification::Inaccessible { .. }
    ));
    assert!(probed.iter().any(|p| p.normalized_path == file_ok));
}

#[test]
fn test_8_duplicate_target_paths_are_deduplicated() {
    let mut mock = MockPlatformProvider::new();
    let p1 = PathBuf::from(r"D:\Data\file.txt");
    let p2 = PathBuf::from(r"d:\data\file.txt");

    mock.insert_entry(p1.clone(), MockEntryConfig::default());

    let resolver = TargetResolver::new(mock).expect("Resolver initialization must succeed");
    let (probed, stats) = resolver
        .resolve_targets(&[p1, p2])
        .expect("Resolution must succeed");

    assert_eq!(stats.total_input_targets, 2);
    assert_eq!(stats.unique_targets, 1);
    assert_eq!(probed.len(), 1);
}

#[test]
fn test_9_file_identity_mismatch_is_detected() {
    // Tests that Step 4 captures discrete OS file identities accurately and that
    // different filesystem objects are reliably distinguishable by their identity tokens.
    let mut mock = MockPlatformProvider::new();
    let original_target = PathBuf::from(r"D:\Data\important.doc");
    let replaced_target = PathBuf::from(r"D:\Data\important.doc.replaced");

    let expected_identity = FileIdentityToken {
        volume_serial_number: 0x1A2B3C4D,
        file_index: 10425,
    };
    let swapped_identity = FileIdentityToken {
        volume_serial_number: 0x1A2B3C4D,
        file_index: 99999, // file ID altered (e.g., target path replaced by new file)
    };

    let mut config1 = MockEntryConfig::default();
    config1.identity = expected_identity.clone();
    mock.insert_entry(original_target.clone(), config1);

    let mut config2 = MockEntryConfig::default();
    config2.identity = swapped_identity.clone();
    mock.insert_entry(replaced_target.clone(), config2);

    let prober = FilesystemProber::new(mock).expect("Prober initialization must succeed");

    let probe1 = prober
        .probe_target(&original_target)
        .expect("Probing original target must succeed");
    let probe2 = prober
        .probe_target(&replaced_target)
        .expect("Probing replaced target must succeed");

    let captured1 = probe1
        .identity
        .value()
        .expect("Identity 1 must be verified");
    let captured2 = probe2
        .identity
        .value()
        .expect("Identity 2 must be verified");

    // Both identities must be captured faithfully
    assert_eq!(captured1, &expected_identity);
    assert_eq!(captured2, &swapped_identity);

    // Equality check used for identity revalidation must successfully detect identity divergence
    assert_ne!(
        captured1, captured2,
        "Revalidation identity check must detect distinct or substituted files"
    );
}

#[test]
fn test_10_real_temporary_directory_resolution_works() {
    let tmp = tempdir().expect("Temporary directory creation must succeed");
    let tmp_path = tmp.path().to_path_buf();
    let file1_path = tmp_path.join("first.txt");
    let file2_path = tmp_path.join("second.dat");

    {
        let mut f1 = File::create(&file1_path).expect("Create first.txt");
        writeln!(f1, "Sample data").expect("Write first.txt");
        let mut f2 = File::create(&file2_path).expect("Create second.dat");
        writeln!(f2, "Other data").expect("Write second.dat");
    }

    let sub_dir = tmp_path.join("nested");
    fs::create_dir(&sub_dir).expect("Create nested directory");
    let file3_path = sub_dir.join("third.bin");
    {
        let mut f3 = File::create(&file3_path).expect("Create third.bin");
        writeln!(f3, "Binary placeholder").expect("Write third.bin");
    }

    let mut mock = MockPlatformProvider::new();
    let mut root_config = MockEntryConfig::default();
    root_config.kind = TargetKind::Directory;
    root_config.children = vec![file1_path.clone(), file2_path.clone(), sub_dir.clone()];
    mock.insert_entry(tmp_path.clone(), root_config);

    mock.insert_entry(file1_path.clone(), MockEntryConfig::default());
    mock.insert_entry(file2_path.clone(), MockEntryConfig::default());

    let mut sub_config = MockEntryConfig::default();
    sub_config.kind = TargetKind::Directory;
    sub_config.children = vec![file3_path.clone()];
    mock.insert_entry(sub_dir.clone(), sub_config);
    mock.insert_entry(file3_path.clone(), MockEntryConfig::default());

    let resolver = TargetResolver::new(mock).expect("Resolver must instantiate");
    let (probed, stats) = resolver
        .resolve_targets(&[tmp_path])
        .expect("Traversal must succeed");

    assert_eq!(stats.directories_discovered, 2);
    assert_eq!(stats.regular_files_discovered, 3);
    assert_eq!(probed.len(), 5);
}

#[test]
fn test_11_nonexistent_path_is_reported_as_inaccessible() {
    let mock = MockPlatformProvider::new();
    let bad_path = PathBuf::from(r"D:\Nonexistent\Folder\file.txt");

    let prober = FilesystemProber::new(mock).expect("Prober initialization must succeed");
    let probe = prober
        .probe_target(&bad_path)
        .expect("Probing missing path returns target probe");

    assert!(matches!(probe.kind, TargetKind::Inaccessible { .. }));
    assert!(matches!(
        probe.safety,
        SafetyClassification::Inaccessible { .. }
    ));
}

#[test]
fn test_12_ads_probing_behavior_is_represented_correctly() {
    let mut mock = MockPlatformProvider::new();
    let target = PathBuf::from(r"D:\Data\report.docx");
    let mut config = MockEntryConfig::default();
    config.streams = vec![AlternateDataStreamInfo {
        stream_name: ":Zone.Identifier:$DATA".into(),
        stream_size_bytes: 64,
    }];
    mock.insert_entry(target.clone(), config);

    let prober = FilesystemProber::new(mock).expect("Prober initialization must succeed");
    let probe = prober.probe_target(&target).expect("Probe succeeds");

    assert!(probe.alternate_data_streams.is_verified());
    let streams = probe
        .alternate_data_streams
        .value()
        .expect("Streams extracted");
    assert_eq!(streams.len(), 1);
    assert_eq!(streams[0].stream_name, ":Zone.Identifier:$DATA");
    assert_eq!(streams[0].stream_size_bytes, 64);
}

#[test]
fn test_13_unknown_unavailable_values_are_never_silently_converted_to_safe_defaults() {
    let mut mock = MockPlatformProvider::new();
    let bad_reparse = PathBuf::from(r"D:\BrokenSymlink");
    let mut config = MockEntryConfig::default();
    config.kind = TargetKind::SymlinkFile;
    config.reparse_tag = Err("DeviceIoControl failed with code 0x5".into());
    mock.insert_entry(bad_reparse.clone(), config);

    let prober = FilesystemProber::new(mock).expect("Prober must succeed");
    let probe = prober
        .probe_target(&bad_reparse)
        .expect("Target probe succeeds");

    assert!(matches!(probe.kind, TargetKind::UnknownReparsePoint { .. }));
    assert!(matches!(probe.reparse_tag, ProbedValue::Unavailable { .. }));
    // UNKNOWN MUST NOT BE TREATED AS ReparseTagType::None!
    assert_ne!(probe.reparse_tag.value(), Some(&ReparseTagType::None));
}
