; Register as a candidate, leaving .pdf's default value and UserChoice alone.
; SHCTX follows Tauri's current-user/all-users install selection.
!define KOMODO_PDF_PROGID "KomodoPDF.AssocFile.PDF"
!define KOMODO_CAPABILITIES "Software\KomodoPDF\Capabilities"

!macro NSIS_HOOK_POSTINSTALL
  WriteRegStr SHCTX "Software\Classes\${KOMODO_PDF_PROGID}" "" "PDF Document"
  WriteRegStr SHCTX "Software\Classes\${KOMODO_PDF_PROGID}\DefaultIcon" "" '$\"$INSTDIR\${MAINBINARYNAME}.exe$\",0'
  WriteRegStr SHCTX "Software\Classes\${KOMODO_PDF_PROGID}\shell\open\command" "" '$\"$INSTDIR\${MAINBINARYNAME}.exe$\" $\"%1$\"'
  WriteRegStr SHCTX "Software\Classes\.pdf\OpenWithProgids" "${KOMODO_PDF_PROGID}" ""
  WriteRegStr SHCTX "${KOMODO_CAPABILITIES}" "ApplicationName" "Komodo PDF"
  WriteRegStr SHCTX "${KOMODO_CAPABILITIES}" "ApplicationDescription" "A local-first, private PDF reader with grounded AI"
  WriteRegStr SHCTX "${KOMODO_CAPABILITIES}" "ApplicationIcon" '$\"$INSTDIR\${MAINBINARYNAME}.exe$\",0'
  WriteRegStr SHCTX "${KOMODO_CAPABILITIES}\FileAssociations" ".pdf" "${KOMODO_PDF_PROGID}"
  WriteRegStr SHCTX "Software\RegisteredApplications" "Komodo PDF" "${KOMODO_CAPABILITIES}"
  System::Call 'shell32::SHChangeNotify(i 0x08000000, i 0, p 0, p 0)'
!macroend

!macro NSIS_HOOK_POSTUNINSTALL
  DeleteRegValue SHCTX "Software\RegisteredApplications" "Komodo PDF"
  DeleteRegKey SHCTX "${KOMODO_CAPABILITIES}"
  DeleteRegKey /ifempty SHCTX "Software\KomodoPDF"
  DeleteRegValue SHCTX "Software\Classes\.pdf\OpenWithProgids" "${KOMODO_PDF_PROGID}"
  DeleteRegKey /ifempty SHCTX "Software\Classes\.pdf\OpenWithProgids"
  DeleteRegKey SHCTX "Software\Classes\${KOMODO_PDF_PROGID}"
  System::Call 'shell32::SHChangeNotify(i 0x08000000, i 0, p 0, p 0)'
!macroend
