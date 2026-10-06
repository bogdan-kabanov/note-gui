import Markdown, { defaultUrlTransform } from "react-markdown";
import remarkGfm from "remark-gfm";

type preview_props = {
  body: string;
  on_open_link: (target_name: string) => void;
};

function prepare_markdown(body: string): string {
  const lines = body.split("\n");
  let in_fence = false;
  return lines
    .map((line) => {
      const trimmed = line.trim();
      if (trimmed.startsWith("```") || trimmed.startsWith("~~~")) {
        in_fence = !in_fence;
        return line;
      }
      if (in_fence) {
        return line;
      }
      return line.replace(/\[\[([^\]\n]+)\]\]/g, (_full, inner: string) => {
        const [target_part, alias_part] = inner.split("|");
        const target = target_part.split("#")[0].split("^")[0].trim();
        const label = (alias_part ?? target_part).trim();
        return `[${label}](note://${encodeURIComponent(target)})`;
      });
    })
    .join("\n");
}

export function MarkdownPreview({ body, on_open_link }: preview_props) {
  return (
    <div className="preview-surface">
      <div className="preview-column">
        <Markdown
          remarkPlugins={[remarkGfm]}
          urlTransform={(url) => (url.startsWith("note:") ? url : defaultUrlTransform(url))}
          components={{
            a: ({ href, children }) => {
              if (href?.startsWith("note://")) {
                const target = decodeURIComponent(href.slice("note://".length));
                return (
                  <button type="button" className="wikilink" onClick={() => on_open_link(target)}>
                    {children}
                  </button>
                );
              }
              return (
                <a href={href} target="_blank" rel="noreferrer">
                  {children}
                </a>
              );
            },
          }}
        >
          {prepare_markdown(body)}
        </Markdown>
      </div>
    </div>
  );
}
