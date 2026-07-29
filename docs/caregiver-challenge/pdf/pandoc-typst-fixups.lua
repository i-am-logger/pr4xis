-- Converts a raw LaTeX \newpage block (from markdown+raw_tex) into a Typst
-- page break, since pandoc's typst writer has no built-in \newpage handling.
function RawBlock(el)
  if el.format == "tex" and el.text:match("^\\newpage%s*$") then
    return pandoc.RawBlock("typst", "#pagebreak()")
  end
  return el
end

-- pandoc's Typst writer emits a bare `#horizontalrule` call for a Markdown
-- `---` thematic break, defined only in its own standalone template --
-- `#include`d fragments get a separate module scope and never see it (typst
-- #include does not inherit #let bindings from the includer). Emit the
-- literal expansion directly instead of relying on an out-of-scope name.
function HorizontalRule()
  return pandoc.RawBlock("typst", "#align(center, line(length: 25%, stroke: 0.5pt))")
end
