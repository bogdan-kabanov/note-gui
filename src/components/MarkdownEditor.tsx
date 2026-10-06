import { markdown } from "@codemirror/lang-markdown";
import { EditorSelection } from "@codemirror/state";
import { Decoration, EditorView, MatchDecorator, ViewPlugin, type ViewUpdate } from "@codemirror/view";
import CodeMirror from "@uiw/react-codemirror";
import { useEffect, useMemo, useRef } from "react";

type editor_props = {
  path: string;
  body: string;
  dark: boolean;
  on_change: (path: string, body: string) => void;
  on_open_link: (target_name: string) => void;
  scroll_line: number | null;
};

const wikilink_decorator = new MatchDecorator({
  regexp: /\[\[[^\]\n]+\]\]/g,
  decoration: Decoration.mark({ class: "cm-wikilink" }),
});

const wikilink_plugin = ViewPlugin.fromClass(
  class {
    decorations;
    constructor(view: EditorView) {
      this.decorations = wikilink_decorator.createDeco(view);
    }
    update(update: ViewUpdate) {
      this.decorations = wikilink_decorator.updateDeco(update, this.decorations);
    }
  },
  { decorations: (value) => value.decorations },
);

function wikilink_at(text: string, offset: number): string | null {
  const pattern = /\[\[([^\]\n]+)\]\]/g;
  let match = pattern.exec(text);
  while (match) {
    const start = match.index;
    const end = start + match[0].length;
    if (offset >= start && offset <= end) {
      return match[1].split("|")[0].split("#")[0].split("^")[0].trim();
    }
    match = pattern.exec(text);
  }
  return null;
}

function editor_theme(dark: boolean) {
  const background = dark ? "#1e1e1e" : "#ffffff";
  const text = dark ? "#dcddde" : "#2e3338";
  const gutter = dark ? "#8d9196" : "#9aa0a6";
  return EditorView.theme(
    {
      "&": { backgroundColor: background, color: text, height: "100%" },
      ".cm-scroller": { fontFamily: "Segoe UI, sans-serif" },
      ".cm-content": { fontSize: "16px", padding: "8px 0 48px", caretColor: text },
      ".cm-gutters": { backgroundColor: background, color: gutter, border: "none" },
      "&.cm-focused": { outline: "none" },
      ".cm-activeLine": { backgroundColor: dark ? "rgba(255,255,255,0.035)" : "rgba(0,0,0,0.03)" },
      ".cm-wikilink": { color: "var(--accent)" },
    },
    { dark },
  );
}

export function MarkdownEditor({ path, body, dark, on_change, on_open_link, scroll_line }: editor_props) {
  const view_ref = useRef<EditorView | null>(null);
  const extensions = useMemo(
    () => [
      markdown(),
      wikilink_plugin,
      editor_theme(dark),
      EditorView.lineWrapping,
      EditorView.domEventHandlers({
        click(event, view) {
          if (!(event.ctrlKey || event.metaKey)) {
            return false;
          }
          const position = view.posAtCoords({ x: event.clientX, y: event.clientY });
          if (position == null) {
            return false;
          }
          const line = view.state.doc.lineAt(position);
          const target = wikilink_at(line.text, position - line.from);
          if (!target) {
            return false;
          }
          event.preventDefault();
          on_open_link(target);
          return true;
        },
      }),
    ],
    [dark, on_open_link],
  );

  useEffect(() => {
    const view = view_ref.current;
    if (!view || scroll_line == null) {
      return;
    }
    if (scroll_line < 1 || scroll_line > view.state.doc.lines) {
      return;
    }
    const line = view.state.doc.line(scroll_line);
    view.dispatch({
      selection: EditorSelection.cursor(line.from),
      scrollIntoView: true,
    });
  }, [scroll_line, path]);

  return (
    <div className="editor-column editor-surface">
      <CodeMirror
        value={body}
        height="100%"
        extensions={extensions}
        basicSetup={{ lineNumbers: true, foldGutter: false }}
        onCreateEditor={(view) => {
          view_ref.current = view;
        }}
        onChange={(value) => {
          if (value !== body) {
            on_change(path, value);
          }
        }}
      />
    </div>
  );
}
