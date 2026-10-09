// A real PDF with alternating page geometry exercises PDF.js layout and workers.
export function fixturePdf(): Uint8Array {
  const objects = ["<< /Type /Catalog /Pages 2 0 R >>", "", "<< /Type /Font /Subtype /Type1 /BaseFont /Helvetica >>"];
  const pages: number[] = [];
  for (let page = 1; page <= 12; page++) {
    const number = objects.length + 1;
    pages.push(number);
    const [width, height] = page % 2 ? [600, 800] : [800, 600];
    objects.push(`<< /Type /Page /Parent 2 0 R /MediaBox [0 0 ${width} ${height}] /Resources << /Font << /F1 3 0 R >> >> /Contents ${number + 1} 0 R >>`);
    const text = `BT /F1 24 Tf 50 ${height - 80} Td (Page ${page}: Komodo PDF scroll regression) Tj ET\n`;
    objects.push(`<< /Length ${text.length} >>\nstream\n${text}endstream`);
  }
  objects[1] = `<< /Type /Pages /Kids [${pages.map((n) => `${n} 0 R`).join(" ")}] /Count 12 >>`;
  let pdf = "%PDF-1.4\n";
  const offsets = [0];
  objects.forEach((body, i) => { offsets.push(pdf.length); pdf += `${i + 1} 0 obj\n${body}\nendobj\n`; });
  const xref = pdf.length;
  pdf += `xref\n0 ${offsets.length}\n0000000000 65535 f \n${offsets.slice(1).map((n) => `${String(n).padStart(10, "0")} 00000 n \n`).join("")}trailer\n<< /Size ${offsets.length} /Root 1 0 R >>\nstartxref\n${xref}\n%%EOF`;
  return new TextEncoder().encode(pdf);
}
