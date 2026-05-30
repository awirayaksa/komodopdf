import * as pdfjsLib from "pdfjs-dist";
import pdfjsWorker from "pdfjs-dist/build/pdf.worker.min.mjs?url";

pdfjsLib.GlobalWorkerOptions.workerSrc = pdfjsWorker;

export async function loadPdfFromArrayBuffer(arrayBuffer: ArrayBuffer) {
  return pdfjsLib.getDocument({ data: arrayBuffer }).promise;
}
