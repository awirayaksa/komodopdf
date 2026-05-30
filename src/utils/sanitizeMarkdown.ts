/**
 * Fixes common LLM-generated Markdown issues so react-markdown / remark
 * can render them properly.
 *
 * Main issues addressed:
 * 1. `** text**` / `**text **` — spaces adjacent to emphasis delimiters
 *    make the run non-left/right-flanking in CommonMark.
 * 2. `****` — empty emphasis runs are invalid.
 * 3. `* text*` / `*text *` — same issue for single-star italic.
 * 4. Emphasis spanning newlines (`**Lists\nare**`) — CommonMark refuses
 *    emphasis across block boundaries, so we split into separate runs.
 * 5. Trailing unbalanced markers during streaming — suppressed to avoid
 *    literal `*` / `**` flashing at the end of partial output.
 */
export function sanitizeMarkdown(text: string): string {
  let result = "";
  let i = 0;

  while (i < text.length) {
    // Bold marker `**`
    if (i + 1 < text.length && text[i] === "*" && text[i + 1] === "*") {
      const next = text.indexOf("**", i + 2);
      if (next === -1) {
        // Unclosed bold at the very end — suppress to avoid literal `**`
        if (i + 2 >= text.length) {
          // suppress trailing `**`
        } else {
          result += "**";
        }
        i += 2;
      } else {
        const inner = text.slice(i + 2, next);
        const trimmed = inner.trim();
        if (trimmed) {
          // Split emphasis that spans newlines into separate bold segments
          const parts = trimmed
            .split("\n")
            .map((s) => s.trim())
            .filter((s) => s);
          if (parts.length > 1) {
            result += parts.map((p) => `**${p}**`).join(" ");
          } else {
            result += `**${trimmed}**`;
          }
        }
        i = next + 2;
      }
      continue;
    }

    // Italic marker `*` — must NOT be part of `**`
    if (text[i] === "*") {
      let next = -1;
      for (let j = i + 1; j < text.length; j++) {
        if (
          text[j] === "*" &&
          text[j - 1] !== "*" &&
          (j + 1 >= text.length || text[j + 1] !== "*")
        ) {
          next = j;
          break;
        }
      }

      if (next === -1) {
        if (i + 1 >= text.length) {
          // suppress trailing `*`
        } else {
          result += "*";
        }
        i += 1;
      } else {
        const inner = text.slice(i + 1, next);
        const trimmed = inner.trim();
        if (trimmed) {
          const parts = trimmed
            .split("\n")
            .map((s) => s.trim())
            .filter((s) => s);
          if (parts.length > 1) {
            result += parts.map((p) => `*${p}*`).join(" ");
          } else {
            result += `*${trimmed}*`;
          }
        }
        i = next + 1;
      }
      continue;
    }

    result += text[i];
    i++;
  }

  return result;
}
