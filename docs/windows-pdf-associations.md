# Windows PDF file opening

Windows installers register Komodo PDF as an available PDF handler. They do not
choose the user's default app. An unpackaged executable does not register itself.

The NSIS hook supports the selected current-user or all-users scope. The MSI WiX
fragment uses the per-machine scope of Tauri's MSI template. Both register the
same application-specific ProgID, `OpenWithProgids`, application capabilities and
`RegisteredApplications`, with a quoted executable and quoted `%1` argument.
Uninstall removes Komodo's own registration. The extension's default value and
Windows `UserChoice` are left alone.

These custom installer definitions are intentional: Tauri 2.11.2's generic NSIS
`fileAssociations` macro writes the extension's default value and does not create
the capabilities registration used here. Do not add a parallel generic `.pdf`
association without reviewing those effects.

## Runtime behavior

Windows single-instance notifications are passed to a serial worker. The synchronous
WM_COPYDATA callback only enqueues requests; file-open events and window activation
run outside the callback to avoid reentering Tauri during window initialization.

The initial process arguments and Windows single-instance callback feed a native
queue. The callback uses the **launching process's** working directory. Native
paths are preserved, including spaces, Unicode, percent signs and `#`; `file:`
URLs are decoded using `url::Url::to_file_path`. Non-file URLs, flags and non-PDF
arguments are ignored. URL query strings and fragments are not supported.

The webview subscribes before draining the queue, so an event sent before React
mounts is harmless. The filesystem plugin grants access only to each requested
file when drained, after plugin initialization. Manual and native requests share
a serialized loader, which hashes bytes before PDF.js transfers them and disposes
of the previous document. This is a single-document reader: a batch leaves the
last successfully opened PDF visible. A failed file does not block the next file.

## Automated checks

```sh
npm ci
npm test
npm run build
python3 scripts/check-windows-associations.py
npx playwright test
cargo test --manifest-path src-tauri/Cargo.toml open_requests
```

The Rust command needs Tauri's platform dependencies. Run it on Windows to execute
the drive-letter, UNC and Windows file-URL test as well. The browser fixtures mock
Tauri IPC while using real PDF.js rendering; they do not exercise Explorer,
WebView2, native IPC or the Windows registry.

## Remaining Windows acceptance checks

Use a test VM and build both installers from this branch with `npm run tauri build`.
Test NSIS current-user and all-users installs, and MSI per-machine installation
separately, including upgrade from 0.1.2 and uninstall/reinstall. Avoid mixing MSI
and NSIS installations in the same scope during these checks.

- Record the existing PDF handler. Install Komodo, check that it is offered in
  Settings > Apps > Default apps and in PDF > Open with. Verify that installation
  has preserved the previous default. Use Open with once for the opening tests.
- With Komodo closed, open a PDF with an ordinary path, then one such as
  `C:\Users\Test User\Documents\日本語 資料\100%20 #1.pdf`. The requested PDF
  should render without choosing a file in Komodo's dialog.
- Repeat with Komodo already running and minimized. The same instance should
  restore, gain focus where Windows permits it, and display the requested PDF.
- Launch multiple PDFs quickly during startup and after startup. Also pass
  several quoted file paths to the executable. Every request should be consumed
  in order and the last successful PDF should remain visible.
- Run the executable from a different working directory with a relative PDF path;
  verify that the sender's directory is used for a second launch.
- Try a `file:///C:/...` URL with encoded spaces/Unicode and an accessible UNC path
  or `file://server/share/...` URL. Network access/permissions are external factors.
- Try an unreadable, deleted or malformed PDF, then a valid PDF. Errors should be
  visible and the next request should work. Confirm the manual Open PDF flow too.
- Uninstall. Komodo's candidate/capability entries should disappear, other PDF
  handlers should remain, and the extension default must not be rewritten by the
  uninstaller. Windows may independently repair a user's choice if it had pointed
  to an application that is now removed.

## Evidence and references

At upstream main `917695075bd0af1be693ed1b78fd66aac33fb5e1`, `lib.rs` registered no
single-instance plugin, never read process arguments, and exposed no file-open
handoff command. PDF opening existed only inside Toolbar's dialog handler.
`tauri.conf.json` contained neither PDF associations nor custom registration.
These omissions prevent the installed application from handling PDFs opened
through Windows file associations.

- [Tauri single-instance lifecycle and callback](https://v2.tauri.app/plugin/single-instance/)
- [Tauri configuration](https://v2.tauri.app/reference/config/)
- [Microsoft application registration](https://learn.microsoft.com/en-us/windows/win32/shell/default-programs#registering-an-application-for-use-with-default-programs)
  describes Capabilities, ProgIDs and RegisteredApplications; its legacy API/UI
  guidance is not a substitute for testing Windows 10/11 Settings.
- [Tauri 2.11.2 NSIS installer](https://github.com/tauri-apps/tauri/blob/tauri-cli-v2.11.2/crates/tauri-bundler/src/bundle/windows/nsis/installer.nsi)
- [Tauri 2.11.2 association macro](https://github.com/tauri-apps/tauri/blob/tauri-cli-v2.11.2/crates/tauri-bundler/src/bundle/windows/nsis/FileAssociation.nsh)
- [Tauri 2.11.2 MSI template](https://github.com/tauri-apps/tauri/blob/tauri-cli-v2.11.2/crates/tauri-bundler/src/bundle/windows/msi/main.wxs)
