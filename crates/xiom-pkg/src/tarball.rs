// Copyright (c) 2026 Eleftherios Notas and The XIOM Authors
// SPDX-License-Identifier: MIT OR Apache-2.0

//! m126: deterministic `.tar.gz` writer.
//!
//! `publish` used to shell out to the system `tar` (or PowerShell
//! `Compress-Archive` on Windows), so the published bytes changed from run to
//! run (entry mtimes, uid/gid, gzip header timestamp, tool-specific
//! extensions) and never matched the release asset built elsewhere. This
//! writer produces byte-identical archives for identical trees:
//!
//! * entries sorted byte-wise by archive path;
//! * ustar headers with `mtime = SOURCE_DATE_EPOCH` (default 0), uid/gid 0,
//!   fixed modes (0644 files / 0755 dirs) and empty uname/gname;
//! * gzip with MTIME 0 and OS 255 containing STORED (uncompressed) deflate
//!   blocks -- no external compressor and deterministic by construction.
//!
//! The stored-deflate choice trades size for reproducibility and zero new
//! dependencies (the workspace supply-chain gate is audited-only). Packages
//! are small; callers that need compression should compress the OUTER
//! transport, not the artifact.

use std::fs;
use std::path::{Path, PathBuf};

use crate::pack::{PackPolicy, Verdict};

/// Block size of the tar format.
const BLOCK: usize = 512;

/// Result of a filtered pack: which entries only landed because `--force`
/// overrode the default refusal list.
#[derive(Debug, Default)]
pub struct PackOutcome {
    pub forced: Vec<String>,
}

/// Read the deterministic mtime: `SOURCE_DATE_EPOCH` seconds when set and
/// valid, otherwise 0 (the reproducible-builds default).
pub fn deterministic_mtime() -> u64 {
    std::env::var("SOURCE_DATE_EPOCH")
        .ok()
        .and_then(|s| s.trim().parse::<u64>().ok())
        .unwrap_or(0)
}

/// Create a deterministic gzipped tarball of `dir` at `output`.
///
/// The archive contains the directory itself (as `dir_name/`) followed by its
/// entries, mirroring `tar -C <parent> <dir_name>`.
#[cfg(test)]
pub fn write_tar_gz(dir: &Path, output: &Path) -> Result<(), String> {
    write_pack(dir, output, deterministic_mtime(), None).map(|_| ())
}

/// Same as [`write_tar_gz`] with an explicit mtime (used by tests).
#[cfg(test)]
pub fn write_tar_gz_with_mtime(dir: &Path, output: &Path, mtime: u64) -> Result<(), String> {
    write_pack(dir, output, mtime, None).map(|_| ())
}

/// REL-20261010-1548-registry-2: the guarded pack used by `xiom pkg publish`.
/// Applies the [`PackPolicy`] while walking: default build/artifact entries
/// fail the pack (unless `--force`), `.xiomignore` entries are skipped, and
/// entries declared in package.xi are kept. Nothing is written when the
/// guard refuses.
pub fn write_tar_gz_filtered(
    dir: &Path,
    output: &Path,
    policy: &PackPolicy,
) -> Result<PackOutcome, String> {
    let outcome = write_pack(dir, output, deterministic_mtime(), Some(policy))?;
    Ok(outcome.unwrap_or_default())
}

fn write_pack(
    dir: &Path,
    output: &Path,
    mtime: u64,
    policy: Option<&PackPolicy>,
) -> Result<Option<PackOutcome>, String> {
    let dir_name = dir
        .file_name()
        .ok_or_else(|| format!("package directory {} has no name", dir.display()))?
        .to_string_lossy()
        .to_string();

    let mut entries: Vec<Entry> = Vec::new();
    let mut refused: Vec<String> = Vec::new();
    let mut forced: Vec<String> = Vec::new();
    collect_entries(dir, &dir_name, "", policy, &mut entries, &mut refused, &mut forced)?;
    if !refused.is_empty() {
        return Err(crate::pack::refusal_message(&refused));
    }
    // Deterministic order: byte-wise path sort (dirs before their contents
    // naturally, since "a" < "a/b").
    entries.sort_by(|a, b| a.archive_path.as_bytes().cmp(b.archive_path.as_bytes()));

    let mut tar: Vec<u8> = Vec::new();
    for entry in &entries {
        append_header(&mut tar, entry, mtime)?;
        if let Some(data) = &entry.data {
            tar.extend_from_slice(data);
            let pad = (BLOCK - (data.len() % BLOCK)) % BLOCK;
            tar.extend(std::iter::repeat(0u8).take(pad));
        }
    }
    // Two zero blocks terminate the archive.
    tar.extend(std::iter::repeat(0u8).take(BLOCK * 2));

    let gz = gzip_stored(&tar);
    fs::write(output, &gz).map_err(|e| format!("cannot write {}: {e}", output.display()))?;
    Ok(policy.map(|_| PackOutcome { forced }))
}

struct Entry {
    archive_path: String,
    is_dir: bool,
    data: Option<Vec<u8>>,
}

fn collect_entries(
    root: &Path,
    prefix: &str,
    rel_base: &str,
    policy: Option<&PackPolicy>,
    out: &mut Vec<Entry>,
    refused: &mut Vec<String>,
    forced: &mut Vec<String>,
) -> Result<(), String> {
    let mut children: Vec<PathBuf> = fs::read_dir(root)
        .map_err(|e| format!("cannot read {}: {e}", root.display()))?
        .map(|r| r.map(|e| e.path()).map_err(|e| e.to_string()))
        .collect::<Result<_, _>>()?;
    children.sort();
    // The directory itself (explicit entry, like tar -C parent dir).
    out.push(Entry {
        archive_path: format!("{prefix}/"),
        is_dir: true,
        data: None,
    });
    for child in children {
        let name = child
            .file_name()
            .ok_or_else(|| format!("entry {} has no name", child.display()))?
            .to_string_lossy()
            .to_string();
        let archive_path = format!("{prefix}/{name}");
        let rel = if rel_base.is_empty() {
            name.clone()
        } else {
            format!("{rel_base}/{name}")
        };
        let meta = fs::symlink_metadata(&child)
            .map_err(|e| format!("cannot stat {}: {e}", child.display()))?;
        if meta.file_type().is_symlink() {
            return Err(format!(
                "refusing to pack symlink {} (packages must contain regular files and directories)",
                child.display()
            ));
        }
        if meta.is_dir() {
            if let Some(policy) = policy {
                match policy.verdict(&rel, true) {
                    Verdict::Skip => continue,
                    Verdict::Refuse => {
                        refused.push(format!("{rel}/"));
                        continue;
                    }
                    Verdict::Keep => {}
                }
                if policy.is_forced() && crate::pack::default_refused(&rel, true) {
                    forced.push(format!("{rel}/"));
                }
            }
            collect_entries(&child, &archive_path, &rel, policy, out, refused, forced)?;
        } else if meta.is_file() {
            if let Some(policy) = policy {
                match policy.verdict(&rel, false) {
                    Verdict::Skip => continue,
                    Verdict::Refuse => {
                        refused.push(rel.clone());
                        continue;
                    }
                    Verdict::Keep => {}
                }
                if policy.is_forced() && crate::pack::default_refused(&rel, false) {
                    forced.push(rel.clone());
                }
            }
            let data = fs::read(&child)
                .map_err(|e| format!("cannot read {}: {e}", child.display()))?;
            out.push(Entry { archive_path, is_dir: false, data: Some(data) });
        } else {
            return Err(format!(
                "refusing to pack special file {} (packages must contain regular files and directories)",
                child.display()
            ));
        }
    }
    Ok(())
}

fn append_header(tar: &mut Vec<u8>, entry: &Entry, mtime: u64) -> Result<(), String> {
    let (name, prefix) = split_ustar_path(&entry.archive_path)?;
    let size = entry.data.as_ref().map(|d| d.len() as u64).unwrap_or(0);
    let mode: u32 = if entry.is_dir { 0o755 } else { 0o644 };
    let typeflag = if entry.is_dir { b'5' } else { b'0' };

    let mut header = [0u8; BLOCK];
    write_bytes(&mut header[0..100], name.as_bytes());
    write_octal(&mut header[100..108], mode as u64);
    write_octal(&mut header[108..116], 0); // uid
    write_octal(&mut header[116..124], 0); // gid
    write_octal(&mut header[124..136], size);
    write_octal(&mut header[136..148], mtime);
    // Checksum placeholder: spaces while computing.
    for b in header[148..156].iter_mut() {
        *b = b' ';
    }
    header[156] = typeflag;
    // linkname 157..257 stays zero, magic/version:
    write_bytes(&mut header[257..263], b"ustar\0");
    write_bytes(&mut header[263..265], b"00");
    // uname/gname/devmajor/devminor stay zero (empty), deterministic.
    write_bytes(&mut header[345..500], prefix.as_bytes());

    let sum: u64 = header.iter().map(|&b| b as u64).sum();
    // 6 octal digits + NUL + space.
    let chk = format!("{sum:06o}\0 ");
    debug_assert_eq!(chk.len(), 8);
    write_bytes(&mut header[148..156], chk.as_bytes());

    tar.extend_from_slice(&header);
    Ok(())
}

/// Split an archive path into ustar `name` (<=100) and `prefix` (<=155).
fn split_ustar_path(path: &str) -> Result<(String, String), String> {
    if path.len() <= 100 {
        return Ok((path.to_string(), String::new()));
    }
    // Find the LAST '/' that leaves a name of at most 100 bytes.
    let bytes = path.as_bytes();
    let mut split = None;
    for i in (0..bytes.len()).rev() {
        if bytes[i] == b'/' && bytes.len() - i - 1 <= 100 && i <= 155 {
            split = Some(i);
            break;
        }
    }
    match split {
        Some(i) => Ok((path[i + 1..].to_string(), path[..i].to_string())),
        None => Err(format!("path too long for ustar: {path}")),
    }
}

fn write_bytes(dst: &mut [u8], src: &[u8]) {
    let n = dst.len().min(src.len());
    dst[..n].copy_from_slice(&src[..n]);
}

fn write_octal(dst: &mut [u8], value: u64) {
    let digits = dst.len() - 1; // NUL terminator
    let s = format!("{value:0width$o}", width = digits);
    write_bytes(dst, s.as_bytes());
}

/// Wrap `data` in a deterministic gzip stream (MTIME 0, OS 255, stored
/// deflate blocks).
pub fn gzip_stored(data: &[u8]) -> Vec<u8> {
    let mut out = Vec::with_capacity(data.len() + data.len() / 1024 * 5 + 32);
    // Header: magic, CM=deflate, FLG=0, MTIME=0, XFL=0, OS=255 (unknown).
    out.extend_from_slice(&[0x1f, 0x8b, 0x08, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0xff]);
    if data.is_empty() {
        // A single final empty stored block.
        out.extend_from_slice(&[0x01, 0x00, 0x00, 0xff, 0xff]);
    } else {
        let mut chunks = data.chunks(65535).peekable();
        while let Some(chunk) = chunks.next() {
            let final_block = chunks.peek().is_none();
            out.push(if final_block { 0x01 } else { 0x00 });
            let len = chunk.len() as u16;
            out.extend_from_slice(&len.to_le_bytes());
            out.extend_from_slice(&(!len).to_le_bytes());
            out.extend_from_slice(chunk);
        }
    }
    out.extend_from_slice(&crc32(data).to_le_bytes());
    out.extend_from_slice(&((data.len() as u32) & 0xffff_ffff).to_le_bytes());
    out
}

/// CRC-32 (IEEE 802.3, reflected) -- bitwise so no table or dependency.
pub fn crc32(data: &[u8]) -> u32 {
    let mut crc: u32 = 0xffff_ffff;
    for &byte in data {
        crc ^= byte as u32;
        for _ in 0..8 {
            let mask = (crc & 1).wrapping_neg();
            crc = (crc >> 1) ^ (0xedb8_8320 & mask);
        }
    }
    !crc
}

#[cfg(test)]
mod tests {
    use super::*;

    fn test_tree(dir: &Path) {
        fs::create_dir_all(dir.join("sub")).unwrap();
        fs::write(dir.join("package.xi"), b"name: demo\nversion: 0.1.0\n").unwrap();
        fs::write(dir.join("sub").join("mod.xi"), b"module demo.sub\n").unwrap();
    }

    fn temp_dir(name: &str) -> PathBuf {
        let dir = std::env::temp_dir().join(format!("xiom_tarball_test_{name}_{}", std::process::id()));
        let _ = fs::remove_dir_all(&dir);
        fs::create_dir_all(&dir).unwrap();
        dir
    }

    #[test]
    fn deterministic_bytes_for_same_tree() {
        let root = temp_dir("det");
        let pkg = root.join("demo");
        test_tree(&pkg);

        let out_a = root.join("a.tar.gz");
        let out_b = root.join("b.tar.gz");
        write_tar_gz_with_mtime(&pkg, &out_a, 0).unwrap();
        write_tar_gz_with_mtime(&pkg, &out_b, 0).unwrap();
        let a = fs::read(&out_a).unwrap();
        let b = fs::read(&out_b).unwrap();
        assert_eq!(a, b, "same tree must produce byte-identical archives");
        assert!(!a.is_empty());
    }

    #[test]
    fn mtimes_do_not_change_the_archive() {
        let root = temp_dir("mtime");
        let pkg = root.join("demo");
        test_tree(&pkg);

        let out_a = root.join("a.tar.gz");
        write_tar_gz_with_mtime(&pkg, &out_a, 0).unwrap();
        // Touch every file with a different mtime, then re-pack.
        let later = std::time::SystemTime::now();
        for f in [pkg.join("package.xi"), pkg.join("sub").join("mod.xi")] {
            let fh = fs::OpenOptions::new().write(true).open(&f).unwrap();
            fh.set_modified(later).unwrap();
        }
        let out_b = root.join("b.tar.gz");
        write_tar_gz_with_mtime(&pkg, &out_b, 0).unwrap();
        assert_eq!(
            fs::read(&out_a).unwrap(),
            fs::read(&out_b).unwrap(),
            "file mtimes must not leak into the archive"
        );
    }

    #[test]
    fn source_date_epoch_changes_only_header_mtime() {
        let root = temp_dir("sde");
        let pkg = root.join("demo");
        test_tree(&pkg);
        let out_a = root.join("a.tar.gz");
        let out_b = root.join("b.tar.gz");
        write_tar_gz_with_mtime(&pkg, &out_a, 0).unwrap();
        write_tar_gz_with_mtime(&pkg, &out_b, 1_700_000_000).unwrap();
        let a = fs::read(&out_a).unwrap();
        let b = fs::read(&out_b).unwrap();
        assert_ne!(a, b, "the header mtime must be recorded");
        assert_eq!(a.len(), b.len());
    }

    #[test]
    fn gzip_trailer_has_crc_and_isize() {
        let data = b"hello deterministic world";
        let gz = gzip_stored(data);
        assert_eq!(&gz[0..4], &[0x1f, 0x8b, 0x08, 0x00]);
        // MTIME 0, OS 255.
        assert_eq!(&gz[4..10], &[0x00, 0x00, 0x00, 0x00, 0x00, 0xff]);
        let n = gz.len();
        let crc = u32::from_le_bytes(gz[n - 8..n - 4].try_into().unwrap());
        let isize = u32::from_le_bytes(gz[n - 4..n].try_into().unwrap());
        assert_eq!(isize as usize, data.len());
        // Independent bitwise CRC check.
        let mut expect: u32 = 0xffff_ffff;
        for &byte in data {
            expect ^= byte as u32;
            for _ in 0..8 {
                expect = if expect & 1 != 0 { (expect >> 1) ^ 0xedb8_8320 } else { expect >> 1 };
            }
        }
        assert_eq!(crc, !expect);
    }

    #[test]
    fn ustar_headers_are_well_formed() {
        let root = temp_dir("ustar");
        let pkg = root.join("demo");
        test_tree(&pkg);
        let out = root.join("a.tar.gz");
        write_tar_gz_with_mtime(&pkg, &out, 0).unwrap();
        let tar = gunzip_stored(&fs::read(&out).unwrap());
        assert_eq!(tar.len() % BLOCK, 0, "tar must be block-aligned");
        // First header: the directory entry.
        assert_eq!(&tar[257..263], b"ustar\0");
        assert_eq!(&tar[263..265], b"00");
        assert_eq!(tar[156], b'5', "first entry is the package directory");
        assert_eq!(&tar[0..5], b"demo/", "entry name carries the package dir");
        assert_eq!(&tar[345..357], &[0u8; 12], "prefix field stays empty");
        // Checksum field: 6 octal digits + NUL + space, and it verifies.
        let chk = std::str::from_utf8(&tar[148..154]).unwrap();
        let parsed = u64::from_str_radix(chk, 8).unwrap();
        let mut copy = tar[0..BLOCK].to_vec();
        for b in copy[148..156].iter_mut() {
            *b = b' ';
        }
        let sum: u64 = copy.iter().map(|&b| b as u64).sum();
        assert_eq!(parsed, sum, "header checksum must verify");
    }

    /// List the ustar member names of a written archive (test helper).
    fn archive_names(path: &Path) -> Vec<String> {
        let tar = gunzip_stored(&fs::read(path).unwrap());
        let mut names = Vec::new();
        let mut off = 0usize;
        while off + BLOCK <= tar.len() {
            if tar[off..off + BLOCK].iter().all(|&b| b == 0) {
                break;
            }
            let name = String::from_utf8_lossy(&tar[off..off + 100])
                .trim_end_matches('\0')
                .to_string();
            let size_str = String::from_utf8_lossy(&tar[off + 124..off + 136])
                .trim_end_matches('\0')
                .trim()
                .to_string();
            let size = usize::from_str_radix(&size_str, 8).unwrap_or(0);
            names.push(name);
            off += BLOCK + size.div_ceil(BLOCK) * BLOCK;
        }
        names
    }

    // REL-20261010-1548-registry-2: the guarded pack refuses build/artifact
    // entries and writes NOTHING on refusal.
    #[test]
    fn m306_filtered_pack_refuses_build_entries() {
        let root = temp_dir("guard_refuse");
        let pkg = root.join("demo");
        test_tree(&pkg);
        fs::create_dir_all(pkg.join("target").join("debug")).unwrap();
        fs::write(pkg.join("target").join("debug").join("junk.o"), b"junk").unwrap();
        fs::write(pkg.join("app.exe"), b"bin").unwrap();

        let policy = PackPolicy::for_package(&pkg, &[], false);
        let out = root.join("a.tar.gz");
        let err = write_tar_gz_filtered(&pkg, &out, &policy).unwrap_err();
        assert!(err.contains("refusing to pack"), "{err}");
        assert!(err.contains("target/"), "{err}");
        assert!(err.contains("app.exe"), "{err}");
        assert!(!out.exists(), "nothing is written when the guard refuses");
    }

    // .xiomignore skips, declared modules are kept, --force packs the rest
    // (and reports what it forced).
    #[test]
    fn m306_filtered_pack_honors_xiomignore_keeps_declared_and_force() {
        let root = temp_dir("guard_keep");
        let pkg = root.join("demo");
        test_tree(&pkg);
        fs::write(pkg.join("notes.tmp"), b"tmp").unwrap();
        fs::write(pkg.join(".xiomignore"), b"*.tmp\n").unwrap();

        let out = root.join("a.tar.gz");
        let policy = PackPolicy::for_package(&pkg, &[], false);
        let outcome = write_tar_gz_filtered(&pkg, &out, &policy).expect("pack");
        assert!(outcome.forced.is_empty(), "no force -> no forced entries");
        let names = archive_names(&out);
        assert!(names.iter().all(|n| !n.contains("notes.tmp")), "{names:?}");
        assert!(names.iter().any(|n| n == "demo/package.xi"), "{names:?}");
        assert!(names.iter().any(|n| n == "demo/sub/mod.xi"), "{names:?}");

        // A module declared in package.xi escapes the refusal list.
        fs::create_dir_all(pkg.join("dist")).unwrap();
        fs::write(pkg.join("dist").join("extra.xi"), b"module extra").unwrap();
        let declared = vec!["dist/extra.xi".to_string()];
        let policy = PackPolicy::for_package(&pkg, &declared, false);
        let out2 = root.join("b.tar.gz");
        write_tar_gz_filtered(&pkg, &out2, &policy).expect("declared pack");
        let names = archive_names(&out2);
        assert!(names.iter().any(|n| n == "demo/dist/extra.xi"), "{names:?}");

        // --force packs refused entries (reported) but .xiomignore stays.
        fs::create_dir_all(pkg.join("target")).unwrap();
        fs::write(pkg.join("target").join("junk.o"), b"junk").unwrap();
        let forced = PackPolicy::for_package(&pkg, &[], true);
        let out3 = root.join("c.tar.gz");
        let outcome = write_tar_gz_filtered(&pkg, &out3, &forced).expect("forced pack");
        assert!(!outcome.forced.is_empty(), "forced entries are reported");
        let names = archive_names(&out3);
        assert!(names.iter().any(|n| n == "demo/target/junk.o"), "{names:?}");
        assert!(names.iter().all(|n| !n.contains("notes.tmp")), "{names:?}");
    }

    // REL-20261014-1504-registry: an ignored default-refused entry is skipped
    // without --force, and --force does not resurrect it.
    #[test]
    fn m306_filtered_pack_xiomignore_suppresses_default_refusals() {
        let root = temp_dir("guard_ignore_defaults");
        let pkg = root.join("demo");
        test_tree(&pkg);
        fs::create_dir_all(pkg.join("target").join("debug")).unwrap();
        fs::write(pkg.join("target").join("debug").join("junk.o"), b"junk").unwrap();
        fs::write(pkg.join("app.exe"), b"bin").unwrap();
        fs::write(pkg.join(".xiomignore"), b"target/\ndist/\n*.exe\n").unwrap();

        let out = root.join("a.tar.gz");
        let policy = PackPolicy::for_package(&pkg, &[], false);
        let outcome = write_tar_gz_filtered(&pkg, &out, &policy).expect("ignored defaults pack");
        assert!(outcome.forced.is_empty());
        let names = archive_names(&out);
        assert!(names.iter().all(|n| !n.contains("target/")), "{names:?}");
        assert!(names.iter().all(|n| !n.ends_with("app.exe")), "{names:?}");
        assert!(names.iter().any(|n| n == "demo/package.xi"), "{names:?}");

        let forced = PackPolicy::for_package(&pkg, &[], true);
        let out2 = root.join("b.tar.gz");
        let outcome = write_tar_gz_filtered(&pkg, &out2, &forced).expect("forced ignored defaults pack");
        assert!(outcome.forced.is_empty(), "ignored entries are not force-reported");
        let names = archive_names(&out2);
        assert!(names.iter().all(|n| !n.contains("target/")), "{names:?}");
        assert!(names.iter().all(|n| !n.ends_with("app.exe")), "{names:?}");

        // A non-ignored refusal still proves --force keeps and reports.
        fs::create_dir_all(pkg.join("build")).unwrap();
        fs::write(pkg.join("build").join("kept.o"), b"obj").unwrap();
        let forced2 = PackPolicy::for_package(&pkg, &[], true);
        let out3 = root.join("c.tar.gz");
        let outcome = write_tar_gz_filtered(&pkg, &out3, &forced2).expect("forced non-ignored pack");
        assert!(outcome.forced.iter().any(|e| e.contains("build/")), "{:?}", outcome.forced);
        assert!(archive_names(&out3).iter().any(|n| n == "demo/build/kept.o"));
    }

    /// Minimal stored-deflate decoder for tests only.
    fn gunzip_stored(gz: &[u8]) -> Vec<u8> {
        assert_eq!(&gz[0..4], &[0x1f, 0x8b, 0x08, 0x00]);
        let mut out = Vec::new();
        let mut i = 10;
        loop {
            let header = gz[i];
            i += 1;
            assert_eq!(header & 0b110, 0, "only stored blocks are emitted");
            let len = u16::from_le_bytes(gz[i..i + 2].try_into().unwrap()) as usize;
            let nlen = u16::from_le_bytes(gz[i + 2..i + 4].try_into().unwrap());
            assert_eq!(!len as u16, nlen);
            i += 4;
            out.extend_from_slice(&gz[i..i + len]);
            i += len;
            if header & 1 == 1 {
                break;
            }
        }
        out
    }
}
