#!/usr/bin/env python3
"""Validate this app's APK metadata, arm64 ELF LOADs and 16KB ZIP alignment.

Uses only the Python standard library plus the Android SDK's aapt executable.
This is a static check, not a substitute for testing on a 16KB-page device.
"""

import argparse
from pathlib import Path, PurePosixPath
import re
import struct
import subprocess
import sys
import zipfile

PAGE_SIZE = 16_384
PT_LOAD = 1
EM_AARCH64 = 183
ET_DYN = 3


class VerificationError(ValueError):
    """An APK property does not meet the project's release requirements."""


def require(condition: bool, message: str) -> None:
    if not condition:
        raise VerificationError(message)


def verify_elf(data: bytes, name: str) -> int:
    """Check all PT_LOAD segments of a little-endian arm64 shared object."""
    require(len(data) >= 64, f"{name}: truncated ELF header")
    require(data[:4] == b"\x7fELF", f"{name}: not an ELF file")
    require(data[4:7] == b"\x02\x01\x01", f"{name}: expected ELF64 little-endian version 1")
    header = struct.unpack_from("<HHIQQQIHHHHHH", data, 16)
    elf_type, machine, version = header[:3]
    ph_offset, eh_size, ph_entry_size, ph_count = header[4], header[7], header[8], header[9]
    require(elf_type == ET_DYN, f"{name}: expected ET_DYN shared object")
    require(machine == EM_AARCH64, f"{name}: expected AArch64 machine ID")
    require(version == 1 and eh_size >= 64, f"{name}: invalid ELF header version/size")
    require(ph_count not in (0, 0xFFFF), f"{name}: missing or unsupported extended program headers")
    require(ph_entry_size >= 56, f"{name}: program-header entries are too small")
    require(ph_offset >= eh_size, f"{name}: program headers overlap ELF header")
    require(ph_offset + ph_entry_size * ph_count <= len(data), f"{name}: truncated program headers")

    load_count = 0
    for index in range(ph_count):
        ph = struct.unpack_from("<IIQQQQQQ", data, ph_offset + index * ph_entry_size)
        segment_type, _, offset, vaddr, _, file_size, memory_size, alignment = ph
        if segment_type != PT_LOAD:
            continue
        load_count += 1
        prefix = f"{name}: PT_LOAD[{index}]"
        require(file_size <= memory_size, f"{prefix}: file size exceeds memory size")
        require(offset + file_size <= len(data), f"{prefix}: file range exceeds ELF size")
        require(alignment >= PAGE_SIZE, f"{prefix}: p_align={alignment} is below 16384")
        require(alignment & (alignment - 1) == 0, f"{prefix}: p_align is not a power of two")
        # ELF requires congruent file/virtual offsets, not that both are zero.
        require(offset % alignment == vaddr % alignment, f"{prefix}: offset/vaddr are not congruent at p_align")
        require(offset % PAGE_SIZE == vaddr % PAGE_SIZE, f"{prefix}: offset/vaddr are not congruent at 16KB")
    require(load_count > 0, f"{name}: no PT_LOAD segments")
    return load_count


def verify_native_libraries(apk: Path) -> tuple[int, int]:
    library_count = 0
    segment_count = 0
    with zipfile.ZipFile(apk) as archive, apk.open("rb") as raw:
        names = [entry.filename for entry in archive.infolist()]
        require(len(names) == len(set(names)), "APK contains duplicate ZIP entry names")
        require("AndroidManifest.xml" in names, "APK has no AndroidManifest.xml")
        for entry in archive.infolist():
            if not entry.filename.endswith(".so"):
                continue
            parts = PurePosixPath(entry.filename).parts
            require(
                len(parts) == 3 and parts[:2] == ("lib", "arm64-v8a"),
                f"Unexpected native library path/ABI: {entry.filename}",
            )
            require(not entry.flag_bits & 1, f"{entry.filename}: encrypted ZIP entry")
            require(
                entry.compress_type in (zipfile.ZIP_STORED, zipfile.ZIP_DEFLATED),
                f"{entry.filename}: unsupported ZIP compression",
            )
            raw.seek(entry.header_offset)
            local_header = raw.read(30)
            require(len(local_header) == 30 and local_header[:4] == b"PK\x03\x04", f"{entry.filename}: invalid local ZIP header")
            name_size, extra_size = struct.unpack_from("<HH", local_header, 26)
            data_offset = entry.header_offset + 30 + name_size + extra_size
            if entry.compress_type == zipfile.ZIP_STORED:
                require(data_offset % PAGE_SIZE == 0, f"{entry.filename}: uncompressed ZIP data is not 16KB-aligned")
            # Compressed .so files are extracted (extractNativeLibs=true), but
            # their ELF LOAD alignment must still support 16KB pages.
            data = archive.read(entry)
            segment_count += verify_elf(data, entry.filename)
            library_count += 1
            print(f"PASS native library: {entry.filename}")
    require(library_count > 0, "APK contains no arm64 shared libraries")
    return library_count, segment_count


def verify_metadata(apk: Path, aapt: Path) -> None:
    result = subprocess.run(
        [str(aapt), "dump", "badging", str(apk)],
        check=True, capture_output=True, text=True, encoding="utf-8", errors="replace",
    )
    badging = result.stdout
    package = re.search(r"^package: name='([^']+)'", badging, re.MULTILINE)
    label = re.search(r"^application-label:'([^']*)'", badging, re.MULTILINE)
    minimum = re.search(r"^sdkVersion:'(\d+)'", badging, re.MULTILINE)
    target = re.search(r"^targetSdkVersion:'(\d+)'", badging, re.MULTILINE)
    require(package is not None and package[1] == "io.github.a2788553793.relaypricecalcm", "Unexpected Android package name")
    require(label is not None and label[1] == "relay-price-calc-m", "Unexpected application label")
    require(minimum is not None and int(minimum[1]) == 26, "Expected minimum SDK 26")
    require(target is not None and int(target[1]) == 34, "Expected target SDK 34")
    permissions = [line for line in badging.splitlines() if line.startswith("uses-permission")]
    require(not permissions, f"Unexpected permissions: {permissions}")
    native_code = re.search(r"^native-code:(.*)$", badging, re.MULTILINE)
    require(
        native_code is not None and re.findall(r"'([^']+)'", native_code[1]) == ["arm64-v8a"],
        "Expected only the arm64-v8a ABI",
    )
    activity = re.search(r"^launchable-activity: name='([^']+)'", badging, re.MULTILINE)
    require(activity is not None and activity[1] == "android.app.NativeActivity", "Expected launchable android.app.NativeActivity")
    print("PASS manifest: expected package/label, API 26/34, arm64 NativeActivity, no requested permissions")


def main() -> int:
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument("apk", type=Path, help="Signed, zipaligned APK to inspect")
    parser.add_argument("--aapt", required=True, type=Path, help="Path to SDK build-tools/35.0.0/aapt")
    args = parser.parse_args()
    try:
        require(args.apk.is_file(), f"APK does not exist: {args.apk}")
        require(args.aapt.is_file(), f"aapt does not exist: {args.aapt}")
        count, segments = verify_native_libraries(args.apk)
        verify_metadata(args.apk, args.aapt)
    except (VerificationError, OSError, ValueError, struct.error, zipfile.BadZipFile, subprocess.CalledProcessError) as error:
        print(f"APK verification FAILED: {error}", file=sys.stderr)
        if isinstance(error, subprocess.CalledProcessError) and error.stderr:
            print(error.stderr, file=sys.stderr)
        return 1
    print(f"PASS: {count} native libraries / {segments} PT_LOAD segments support 16KB alignment")
    print("Static verification only; run the app on real Android devices before release.")
    return 0


if __name__ == "__main__":
    raise SystemExit(main())
