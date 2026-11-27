#!/usr/bin/env python3
"""Synthetic APK verifier tests, not Android installation tests."""
import contextlib
import importlib.util
import io
from pathlib import Path
import struct
import tempfile
import unittest
from unittest.mock import patch
import zipfile

spec = importlib.util.spec_from_file_location("verify_apk", Path(__file__).with_name("verify-apk.py"))
v = importlib.util.module_from_spec(spec)
spec.loader.exec_module(v)

def elf(align=16384, machine=183, vaddr=0):
    ident = bytes([127, 69, 76, 70, 2, 1, 1]) + bytes(9)
    return ident + struct.pack("<HHIQQQIHHHHHH", 3, machine, 1, 0, 64, 0, 0, 64, 56, 1, 0, 0, 0) + struct.pack("<IIQQQQQQ", 1, 5, 0, vaddr, 0, 120, 120, align)

BADGING = """package: name='io.github.a2788553793.relaypricecalcm'
application-label:'relay-price-calc-m'
sdkVersion:'26'
targetSdkVersion:'34'
native-code: 'arm64-v8a'
launchable-activity: name='android.app.NativeActivity'
"""

class Tests(unittest.TestCase):
    def test_elf(self):
        self.assertEqual(v.verify_elf(elf(), "fixture"), 1)
        for data in (b"short", b"!" + elf()[1:], elf(4096), elf(24576), elf(machine=62), elf(vaddr=1)):
            with self.subTest(data=data[:16]), self.assertRaises(v.VerificationError):
                v.verify_elf(data, "fixture")

    def test_zip(self):
        with tempfile.TemporaryDirectory() as tmp:
            apk = Path(tmp) / "fixture.apk"
            for abi, stored, aligned, good in (("arm64-v8a", False, False, True), ("arm64-v8a", True, True, True), ("x86_64", False, False, False), ("arm64-v8a", True, False, False)):
                with zipfile.ZipFile(apk, "w") as z:
                    z.writestr("AndroidManifest.xml", b"synthetic fixture")
                    item = zipfile.ZipInfo(f"lib/{abi}/libfixture.so")
                    item.compress_type = zipfile.ZIP_STORED if stored else zipfile.ZIP_DEFLATED
                    if aligned:
                        size = (-(z.fp.tell() + 30 + len(item.filename.encode()))) % 16384
                        item.extra = struct.pack("<HH", 0xCAFE, size - 4) + bytes(size - 4)
                    z.writestr(item, elf())
                with self.subTest(abi=abi, stored=stored, aligned=aligned), contextlib.redirect_stdout(io.StringIO()):
                    if good:
                        self.assertEqual(v.verify_native_libraries(apk), (1, 1))
                    else:
                        with self.assertRaises(v.VerificationError):
                            v.verify_native_libraries(apk)

    def metadata(self, text):
        result = type("Result", (), {"stdout": text})()
        with patch.object(v.subprocess, "run", return_value=result), contextlib.redirect_stdout(io.StringIO()):
            v.verify_metadata(Path("fixture.apk"), Path("fixture-aapt"))

    def test_metadata(self):
        self.metadata(BADGING)
        for text in (BADGING.replace("relaypricecalcm", "wrong"), BADGING.replace("relay-price-calc-m", "wrong"), BADGING.replace("sdkVersion:'26'", "sdkVersion:'24'"), BADGING.replace("targetSdkVersion:'34'", "targetSdkVersion:'35'"), BADGING.replace("arm64-v8a", "x86_64"), BADGING.replace("android.app.NativeActivity", "wrong.Activity"), BADGING + "uses-permission: name='android.permission.INTERNET'\n"):
            with self.subTest(text=text), self.assertRaises(v.VerificationError):
                self.metadata(text)

if __name__ == "__main__":
    unittest.main()
