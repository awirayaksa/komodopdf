"""Static installer checks; never reads or writes the Windows registry."""
import json
from pathlib import Path
import re
import unittest
import xml.etree.ElementTree as ET

ROOT = Path(__file__).resolve().parents[1] / "src-tauri"
NS = {"w": "http://schemas.microsoft.com/wix/2006/wi"}
PROGID = "KomodoPDF.AssocFile.PDF"
CAPABILITIES = r"Software\KomodoPDF\Capabilities"


class WindowsAssociations(unittest.TestCase):
    def setUp(self):
        self.config = json.loads((ROOT / "tauri.conf.json").read_text())
        self.windows = self.config["bundle"]["windows"]
        self.nsi = (ROOT / self.windows["nsis"]["installerHooks"]).read_text()
        self.wix = ET.parse(ROOT / self.windows["wix"]["fragmentPaths"][0])
        self.registry = {}

        def visit(element, root="", key=""):
            if element.tag == "{" + NS["w"] + "}RegistryKey":
                root = element.get("Root", root)
                key = (key + "\\" if key else "") + element.get("Key")
            if element.tag == "{" + NS["w"] + "}RegistryValue":
                self.registry[(root, key, element.get("Name", ""))] = element.get("Value")
            for child in element:
                visit(child, root, key)

        visit(self.wix.getroot())

    def test_both_installer_targets_include_registration(self):
        self.assertEqual(set(self.config["bundle"]["targets"]), {"nsis", "msi"})
        self.assertEqual(self.windows["nsis"]["installMode"], "both")
        component = self.wix.find(".//w:Component", NS)
        self.assertIn(component.get("Id"), self.windows["wix"]["componentRefs"])
        # Tauri's generic NSIS association macro changes the extension default.
        # Use these candidate-only hooks/fragments instead.
        self.assertFalse(self.config["bundle"].get("fileAssociations"))
        self.assertIn("!macro NSIS_HOOK_POSTINSTALL", self.nsi)
        self.assertIn("!macro NSIS_HOOK_POSTUNINSTALL", self.nsi)

    def test_msi_has_capabilities_and_a_quoted_launch_command(self):
        self.assertEqual(self.registry[("HKLM", r"Software\RegisteredApplications", "Komodo PDF")], CAPABILITIES)
        self.assertEqual(self.registry[("HKLM", CAPABILITIES + r"\FileAssociations", ".pdf")], PROGID)
        self.assertTrue(self.registry[("HKLM", CAPABILITIES, "ApplicationDescription")])
        self.assertEqual(self.registry[("HKLM", "Software\\Classes\\" + PROGID + r"\shell\open\command", "")], '"[INSTALLDIR]komodo-pdf.exe" "%1"')
        self.assertIn(("HKLM", r"Software\Classes\.pdf\OpenWithProgids", PROGID), self.registry)

    def test_msi_avoids_cross_feature_file_references(self):
        self.assertNotIn("[#Path]", ET.tostring(self.wix.getroot(), encoding="unicode"))


    def test_nsis_scopes_registration_and_unregistration_to_selected_install_mode(self):
        self.assertIn(f'!define KOMODO_PDF_PROGID "{PROGID}"', self.nsi)
        self.assertIn(f'!define KOMODO_CAPABILITIES "{CAPABILITIES}"', self.nsi)
        writes = [line.strip() for line in self.nsi.splitlines() if line.strip().startswith("WriteReg")]
        self.assertTrue(writes)
        self.assertTrue(all(line.startswith("WriteRegStr SHCTX ") for line in writes))
        self.assertIn("'$\\\"$INSTDIR\\${MAINBINARYNAME}.exe$\\\" $\\\"%1$\\\"'", self.nsi)
        self.assertIn('WriteRegStr SHCTX "Software\\RegisteredApplications" "Komodo PDF" "${KOMODO_CAPABILITIES}"', self.nsi)
        self.assertIn('WriteRegStr SHCTX "${KOMODO_CAPABILITIES}\\FileAssociations" ".pdf" "${KOMODO_PDF_PROGID}"', self.nsi)
        self.assertIn('DeleteRegValue SHCTX "Software\\Classes\\.pdf\\OpenWithProgids" "${KOMODO_PDF_PROGID}"', self.nsi)
        self.assertIn('DeleteRegValue SHCTX "Software\\RegisteredApplications" "Komodo PDF"', self.nsi)

    def test_neither_installer_overwrites_the_pdf_default_or_user_choice(self):
        self.assertFalse(any(key.lower() == r"software\classes\.pdf" and name == "" for _, key, name in self.registry))
        self.assertNotIn("UserChoice", "\n".join(line for line in self.nsi.splitlines() if not line.lstrip().startswith(";")))
        self.assertNotIn("UserChoice", str(self.registry))
        self.assertNotRegex(self.nsi, r'(?:WriteReg\w+|DeleteRegKey)\s+SHCTX\s+"Software\\Classes\\\.pdf"')
        self.assertNotIn("APP_ASSOCIATE", self.nsi)


if __name__ == "__main__":
    unittest.main(verbosity=2)
