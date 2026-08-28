import "./wallpaper.css";

import React from "react";
import ReactDOM from "react-dom/client";

const BUTTON_NAMES = [
  "Left",
  "Middle",
  "Right",
  "Back",
  "Forward",
];
const BUTTON_COLORS = [
  "#34a0a6",
  "#acb06b",
  "#c9628f",
  "#8862c9",
  "#667eea",
];

const App = () => {
  const [clock, setClock] = React.useState("");
  const [position, setPosition] = React.useState({
    x: 0,
    y: 0,
  });
  const [moveRate, setMoveRate] = React.useState(0);
  const [counters, setCounters] = React.useState({
    moves: 0,
    clicks: 0,
    keys: 0,
  });
  const [buttonsDown, setButtonsDown] = React.useState<number[]>([]);
  const [lastKey, setLastKey] = React.useState("—");
  const [keyHistory, setKeyHistory] = React.useState<
    {
      id: number;
      label: string;
    }[]
  >([]);
  const keyId = React.useRef(0);

  const surfaceRef = React.useRef<HTMLDivElement>(null);
  const crosshairRef = React.useRef<HTMLDivElement>(null);
  const moveCount = React.useRef(0);
  const totals = React.useRef({
    moves: 0,
    clicks: 0,
    keys: 0,
  });

  React.useEffect(() => {
    const tick = () => setClock(new Date().toLocaleTimeString());
    tick();
    const timer = setInterval(tick, 1000);

    // Readouts refresh on a timer so mousemove floods never re-render React.
    const stats = setInterval(() => {
      setMoveRate(moveCount.current * 2);
      moveCount.current = 0;
      setCounters({
        ...totals.current,
      });
    }, 500);

    const ripple = (x: number, y: number, color: string, label: string) => {
      const surface = surfaceRef.current;
      if (!surface) return;
      const el = document.createElement("div");
      el.className = "ripple";
      el.style.left = `${x}px`;
      el.style.top = `${y}px`;
      el.style.borderColor = color;
      el.textContent = label;
      surface.appendChild(el);
      setTimeout(() => el.remove(), 900);
    };

    const onMouseMove = (e: MouseEvent) => {
      moveCount.current += 1;
      totals.current.moves += 1;
      const crosshair = crosshairRef.current;
      if (crosshair) {
        crosshair.style.transform = `translate(${e.clientX}px, ${e.clientY}px)`;
      }
      setPosition({
        x: e.clientX,
        y: e.clientY,
      });
    };

    const onMouseDown = (e: MouseEvent) => {
      totals.current.clicks += 1;
      setButtonsDown((down) =>
        down.includes(e.button)
          ? down
          : [
              ...down,
              e.button,
            ],
      );
      ripple(e.clientX, e.clientY, BUTTON_COLORS[e.button] ?? "#fff", BUTTON_NAMES[e.button] ?? `#${e.button}`);
    };

    const onMouseUp = (e: MouseEvent) => {
      setButtonsDown((down) => down.filter((b) => b !== e.button));
    };

    const onKeyDown = (e: KeyboardEvent) => {
      if (e.repeat) return;
      totals.current.keys += 1;
      const label = e.key === " " ? "Space" : e.key;
      setLastKey(label);
      keyId.current += 1;
      setKeyHistory((history) =>
        [
          {
            id: keyId.current,
            label,
          },
          ...history,
        ].slice(0, 12),
      );
    };

    window.addEventListener("mousemove", onMouseMove);
    window.addEventListener("mousedown", onMouseDown);
    window.addEventListener("mouseup", onMouseUp);
    window.addEventListener("keydown", onKeyDown);

    return () => {
      clearInterval(timer);
      clearInterval(stats);
      window.removeEventListener("mousemove", onMouseMove);
      window.removeEventListener("mousedown", onMouseDown);
      window.removeEventListener("mouseup", onMouseUp);
      window.removeEventListener("keydown", onKeyDown);
    };
  }, []);

  return (
    <div
      className="surface"
      ref={surfaceRef}
    >
      <div
        className="crosshair"
        ref={crosshairRef}
      />

      <header>
        <h1>Input Test Surface</h1>
        <p>
          Attach with <strong>forward mouse / keyboard</strong>, then click the desktop (or press Win+D) — input is
          forwarded only while the desktop is focused.
        </p>
        <div className="clock">{clock}</div>
      </header>

      <div className="panels">
        <section>
          <h2>Mouse</h2>
          <div className="readout">
            <span className="value">
              {position.x}, {position.y}
            </span>
            <span className="label">position</span>
          </div>
          <div className="readout">
            <span className="value">{moveRate}</span>
            <span className="label">moves / sec</span>
          </div>
          <div className="buttons">
            {BUTTON_NAMES.map((name, index) => (
              <span
                className={buttonsDown.includes(index) ? "button down" : "button"}
                key={name}
                style={{
                  borderColor: BUTTON_COLORS[index],
                }}
              >
                {name}
              </span>
            ))}
          </div>
        </section>

        <section>
          <h2>Keyboard</h2>
          <div className="last-key">{lastKey}</div>
          <div className="key-history">
            {keyHistory.map((entry) => (
              <span
                className="key"
                key={entry.id}
              >
                {entry.label}
              </span>
            ))}
          </div>
        </section>

        <section>
          <h2>Totals</h2>
          <div className="readout">
            <span className="value">{counters.moves}</span>
            <span className="label">mouse moves</span>
          </div>
          <div className="readout">
            <span className="value">{counters.clicks}</span>
            <span className="label">clicks</span>
          </div>
          <div className="readout">
            <span className="value">{counters.keys}</span>
            <span className="label">keys</span>
          </div>
        </section>
      </div>
    </div>
  );
};

const rootElement = document.getElementById("root");

if (!rootElement) throw new Error("Failed to find the root element");

const root = ReactDOM.createRoot(rootElement);

root.render(
  <React.StrictMode>
    <App />
  </React.StrictMode>,
);
