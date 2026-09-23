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

/// Block size of the tar format.
const BLOCK: usize = 512;

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
pub fn write_tar_gz(dir: &Path, output: &Path) -> Result<(), String> {
    write_tar_gz_with_mtime(dir, output, deterministic_mtime())
}

/// Same as [`write_tar_gz`] with an explicit mtime (used by tests).
pub fn write_tar_gz_with_mtime(dir: &Path, output: &Path, mtime: u64) -> Result<(), String> {
    let dir_name = dir
        .file_name()
        .ok_or_else(|| format!("package directory {} has no name", dir.display()))?
        .to_string_lossy()
        .to_string();

    let mut entries: Vec<Entry> = Vec::new();
    collect_entries(dir, &dir_name, &mut entries)?;
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
    Ok(())
}

struct Entry {
    archive_path: String,
    is_dir: bool,
    data: Option<Vec<u8>>,
}

fn collect_entries(root: &Path, prefix: &str, out: &mut Vec<Entry>) -> Result<(), String> {
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
        let meta = fs::symlink_metadata(&child)
            .map_err(|e| format!("cannot stat {}: {e}", child.display()))?;
        if meta.file_type().is_symlink() {
            return Err(format!(
                "refusing to pack symlink {} (packages must contain regular files and directories)",
                child.display()
            ));
        }
        if meta.is_dir() {
            collect_entries(&child, &archive_path, out)?;
        } else if meta.is_file() {
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
