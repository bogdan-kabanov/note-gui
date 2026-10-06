import { useEffect, useRef } from "react";
import { use_app } from "../state/use_app";

type sim_node = {
  path: string;
  title: string;
  x: number;
  y: number;
  vx: number;
  vy: number;
};

export function GraphView() {
  const app = use_app();
  const canvas_ref = useRef<HTMLCanvasElement | null>(null);
  const nodes_ref = useRef<sim_node[]>([]);
  const drag_ref = useRef<sim_node | null>(null);

  useEffect(() => {
    const canvas = canvas_ref.current;
    const graph = app.graph;
    if (!canvas || !graph) {
      return;
    }
    const context = canvas.getContext("2d");
    if (!context) {
      return;
    }
    const width = canvas.clientWidth;
    const height = canvas.clientHeight;
    const ratio = window.devicePixelRatio || 1;
    canvas.width = Math.max(1, Math.floor(width * ratio));
    canvas.height = Math.max(1, Math.floor(height * ratio));
    context.setTransform(ratio, 0, 0, ratio, 0, 0);
    const styles = getComputedStyle(document.documentElement);
    const accent = styles.getPropertyValue("--accent").trim() || "#7c6aef";
    const text = styles.getPropertyValue("--text-normal").trim();
    const muted = styles.getPropertyValue("--text-muted").trim();
    const nodes: sim_node[] = graph.nodes.map((node, index) => {
      const angle = (Math.PI * 2 * index) / Math.max(graph.nodes.length, 1);
      const radius = Math.min(width, height) * 0.28;
      return {
        path: node.path,
        title: node.title,
        x: width / 2 + Math.cos(angle) * radius,
        y: height / 2 + Math.sin(angle) * radius,
        vx: 0,
        vy: 0,
      };
    });
    nodes_ref.current = nodes;
    const index_by_path = new Map(nodes.map((node, index) => [node.path, index]));
    let frame = 0;
    let animation = 0;
    const step = () => {
      if (frame < 220) {
        for (let first = 0; first < nodes.length; first += 1) {
          for (let second = first + 1; second < nodes.length; second += 1) {
            const left = nodes[first];
            const right = nodes[second];
            let dx = left.x - right.x;
            let dy = left.y - right.y;
            let distance = Math.hypot(dx, dy) || 0.1;
            const force = 900 / (distance * distance);
            dx = (dx / distance) * force;
            dy = (dy / distance) * force;
            left.vx += dx;
            left.vy += dy;
            right.vx -= dx;
            right.vy -= dy;
          }
        }
        for (const edge of graph.edges) {
          const source = index_by_path.get(edge.source_path);
          const target = index_by_path.get(edge.target_path);
          if (source == null || target == null) {
            continue;
          }
          const left = nodes[source];
          const right = nodes[target];
          const dx = right.x - left.x;
          const dy = right.y - left.y;
          const distance = Math.hypot(dx, dy) || 0.1;
          const pull = (distance - 180) * 0.02;
          left.vx += (dx / distance) * pull;
          left.vy += (dy / distance) * pull;
          right.vx -= (dx / distance) * pull;
          right.vy -= (dy / distance) * pull;
        }
        for (const node of nodes) {
          if (drag_ref.current === node) {
            node.vx = 0;
            node.vy = 0;
            continue;
          }
          node.vx += (width / 2 - node.x) * 0.004;
          node.vy += (height / 2 - node.y) * 0.004;
          node.vx *= 0.82;
          node.vy *= 0.82;
          node.x += node.vx;
          node.y += node.vy;
        }
        frame += 1;
      }
      context.clearRect(0, 0, width, height);
      context.strokeStyle = muted;
      context.lineWidth = 1;
      for (const edge of graph.edges) {
        const source = index_by_path.get(edge.source_path);
        const target = index_by_path.get(edge.target_path);
        if (source == null || target == null) {
          continue;
        }
        context.beginPath();
        context.moveTo(nodes[source].x, nodes[source].y);
        context.lineTo(nodes[target].x, nodes[target].y);
        context.stroke();
      }
      for (const node of nodes) {
        context.beginPath();
        context.fillStyle = accent;
        context.arc(node.x, node.y, 7, 0, Math.PI * 2);
        context.fill();
        context.fillStyle = text;
        context.font = "12px Segoe UI";
        context.fillText(node.title, node.x + 10, node.y + 4);
      }
      if (frame < 220) {
        animation = requestAnimationFrame(step);
      }
    };
    animation = requestAnimationFrame(step);
    return () => cancelAnimationFrame(animation);
  }, [app.graph, app.resolved_theme]);

  function node_at(x: number, y: number): sim_node | null {
    return nodes_ref.current.find((node) => Math.hypot(node.x - x, node.y - y) < 12) ?? null;
  }

  return (
    <div className="graph-surface panel-body">
      {app.graph && app.graph.nodes.length > 0 ? (
        <canvas
          ref={canvas_ref}
          onPointerDown={(event) => {
            const rect = event.currentTarget.getBoundingClientRect();
            drag_ref.current = node_at(event.clientX - rect.left, event.clientY - rect.top);
          }}
          onPointerMove={(event) => {
            if (!drag_ref.current) {
              return;
            }
            const rect = event.currentTarget.getBoundingClientRect();
            drag_ref.current.x = event.clientX - rect.left;
            drag_ref.current.y = event.clientY - rect.top;
          }}
          onPointerUp={(event) => {
            const dragged = drag_ref.current;
            drag_ref.current = null;
            if (!dragged) {
              return;
            }
            const rect = event.currentTarget.getBoundingClientRect();
            const found = node_at(event.clientX - rect.left, event.clientY - rect.top);
            if (found && found.path === dragged.path) {
              app.set_center_view("editor");
              void app.open_note(found.path);
            }
          }}
        />
      ) : (
        <div className="graph-empty">
          <h2>Граф пуст</h2>
          <p className="empty-copy">Связи появятся, когда в заметках будут вики-ссылки.</p>
        </div>
      )}
    </div>
  );
}
