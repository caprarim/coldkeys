import { useEffect, useState } from "react";
import { invoke } from "@tauri-apps/api/core";
import { open as openDialog } from "@tauri-apps/plugin-dialog";

const MODIFIER_CODES = [
  "ControlLeft",
  "ControlRight",
  "ShiftLeft",
  "ShiftRight",
  "AltLeft",
  "AltRight",
  "MetaLeft",
  "MetaRight",
];

function codeToKey(code) {
  if (code.startsWith("Key")) return code.slice(3);
  if (code.startsWith("Digit")) return code.slice(5);
  if (code.startsWith("Numpad")) return code;
  return code;
}

function eventToAccelerator(event) {
  if (MODIFIER_CODES.includes(event.code)) return null;
  const parts = [];
  if (event.ctrlKey) parts.push("Control");
  if (event.shiftKey) parts.push("Shift");
  if (event.altKey) parts.push("Alt");
  if (event.metaKey) parts.push("Super");
  parts.push(codeToKey(event.code));
  return parts.join("+");
}

function prettyAccelerator(accel) {
  if (!accel) return "Not set";
  return accel
    .split("+")
    .map((part) => (part === "Control" ? "Ctrl" : part))
    .join(" + ");
}

function emptyBind() {
  return {
    id: crypto.randomUUID().slice(0, 8),
    name: "",
    command: "",
    accelerator: "",
    enabled: true,
    gnome_key: null,
  };
}

function KeyRecorder({ value, onChange }) {
  const [recording, setRecording] = useState(false);

  useEffect(() => {
    if (!recording) return;
    function handler(event) {
      event.preventDefault();
      if (event.code === "Escape") {
        setRecording(false);
        return;
      }
      const accel = eventToAccelerator(event);
      if (accel) {
        onChange(accel);
        setRecording(false);
      }
    }
    window.addEventListener("keydown", handler, true);
    return () => window.removeEventListener("keydown", handler, true);
  }, [recording, onChange]);

  return (
    <button
      type="button"
      onClick={() => setRecording(true)}
      className={`w-full rounded-lg border px-3 py-2 text-left text-sm ${
        recording
          ? "border-amber-500 bg-amber-500/10 text-amber-200"
          : "border-neutral-700 bg-neutral-900 text-neutral-200 hover:border-neutral-500"
      }`}
    >
      {recording ? "Press your keys now, Esc to cancel" : prettyAccelerator(value)}
    </button>
  );
}

function Editor({ bind, onSave, onCancel, error }) {
  const [draft, setDraft] = useState(bind);

  async function browse() {
    const picked = await openDialog({ multiple: false, directory: false });
    if (picked) setDraft({ ...draft, command: picked });
  }

  return (
    <div className="fixed inset-0 z-20 flex items-center justify-center bg-black/70 p-4">
      <div className="w-full max-w-md rounded-xl border border-neutral-700 bg-neutral-900 p-5">
        <h2 className="mb-4 text-base font-semibold text-neutral-100">
          {bind.name ? "Edit shortcut" : "New shortcut"}
        </h2>

        <label className="mb-1 block text-xs text-neutral-400">App name</label>
        <input
          value={draft.name}
          onChange={(e) => setDraft({ ...draft, name: e.target.value })}
          placeholder="ColdShot"
          className="mb-4 w-full rounded-lg border border-neutral-700 bg-neutral-950 px-3 py-2 text-sm text-neutral-100 outline-none focus:border-neutral-500"
        />

        <label className="mb-1 block text-xs text-neutral-400">Command to run</label>
        <div className="mb-4 flex gap-2">
          <input
            value={draft.command}
            onChange={(e) => setDraft({ ...draft, command: e.target.value })}
            placeholder="/usr/bin/coldshot"
            className="min-w-0 flex-1 rounded-lg border border-neutral-700 bg-neutral-950 px-3 py-2 text-sm text-neutral-100 outline-none focus:border-neutral-500"
          />
          <button
            type="button"
            onClick={browse}
            className="shrink-0 rounded-lg border border-neutral-700 px-3 py-2 text-sm text-neutral-300 hover:border-neutral-500"
          >
            Browse
          </button>
        </div>

        <label className="mb-1 block text-xs text-neutral-400">Keybind</label>
        <div className="mb-5">
          <KeyRecorder value={draft.accelerator} onChange={(accelerator) => setDraft({ ...draft, accelerator })} />
        </div>

        {error ? (
          <p className="mb-4 rounded-lg border border-red-900 bg-red-950/60 px-3 py-2 text-xs text-red-300">{error}</p>
        ) : null}

        <div className="flex justify-end gap-2">
          <button
            type="button"
            onClick={onCancel}
            className="rounded-lg px-4 py-2 text-sm text-neutral-400 hover:text-neutral-200"
          >
            Cancel
          </button>
          <button
            type="button"
            disabled={!draft.name.trim() || !draft.command.trim() || !draft.accelerator}
            onClick={() => onSave(draft)}
            className="rounded-lg bg-neutral-100 px-4 py-2 text-sm font-medium text-neutral-900 disabled:cursor-not-allowed disabled:bg-neutral-700 disabled:text-neutral-500"
          >
            Save
          </button>
        </div>
      </div>
    </div>
  );
}

export default function App() {
  const [binds, setBinds] = useState([]);
  const [editing, setEditing] = useState(null);
  const [error, setError] = useState("");
  const [notice, setNotice] = useState("");
  const [os, setOs] = useState("");

  useEffect(() => {
    invoke("list_binds").then(setBinds);
    invoke("platform").then(setOs);
  }, []);

  async function save(draft) {
    try {
      const next = await invoke("upsert_bind", { bind: draft });
      setBinds(next);
      setEditing(null);
      setError("");
    } catch (e) {
      setError(String(e));
    }
  }

  async function remove(bind) {
    const ok = window.confirm(`Delete the shortcut for ${bind.name}?`);
    if (!ok) return;
    const next = await invoke("delete_bind", { id: bind.id });
    setBinds(next);
  }

  async function importSystem() {
    try {
      const next = await invoke("import_system");
      setBinds(next);
      setNotice("System shortcuts imported");
    } catch (e) {
      setNotice(String(e));
    }
  }

  async function run(bind) {
    try {
      await invoke("run_bind", { id: bind.id });
      setNotice(`Launched ${bind.name}`);
    } catch (e) {
      setNotice(String(e));
    }
  }

  useEffect(() => {
    if (!notice) return;
    const timer = setTimeout(() => setNotice(""), 2600);
    return () => clearTimeout(timer);
  }, [notice]);

  return (
    <div className="flex h-full flex-col">
      <header className="flex items-center justify-between border-b border-neutral-800 px-5 py-4">
        <div>
          <h1 className="text-lg font-semibold text-neutral-100">ColdKeys</h1>
          <p className="text-xs text-neutral-500">
            {binds.length} shortcut{binds.length === 1 ? "" : "s"}
            {os ? ` on ${os}` : ""}
          </p>
        </div>
        <div className="flex gap-2">
          {os === "linux" ? (
            <button
              type="button"
              onClick={importSystem}
              className="rounded-lg border border-neutral-700 px-3 py-2 text-sm text-neutral-300 hover:border-neutral-500"
            >
              Import system shortcuts
            </button>
          ) : null}
          <button
            type="button"
            onClick={() => setEditing(emptyBind())}
            className="rounded-lg bg-neutral-100 px-4 py-2 text-sm font-medium text-neutral-900 hover:bg-white"
          >
            Add app
          </button>
        </div>
      </header>

      {notice ? (
        <p className="border-b border-neutral-800 bg-neutral-900 px-5 py-2 text-xs text-neutral-300">{notice}</p>
      ) : null}

      <main className="flex-1 overflow-y-auto px-5 py-4">
        {binds.length === 0 ? (
          <p className="mt-16 text-center text-sm text-neutral-500">
            No shortcuts yet. Add an app to get started.
          </p>
        ) : (
          <ul className="flex flex-col gap-2">
            {binds.map((bind) => (
              <li
                key={bind.id}
                className="flex items-center gap-4 rounded-xl border border-neutral-800 bg-neutral-900/60 px-4 py-3"
              >
                <div className="min-w-0 flex-1">
                  <p className="truncate text-sm font-medium text-neutral-100">{bind.name}</p>
                  <p className="truncate text-xs text-neutral-500">{bind.command}</p>
                </div>

                <span className="shrink-0 rounded-md border border-neutral-700 bg-neutral-950 px-2.5 py-1 font-mono text-xs text-neutral-300">
                  {prettyAccelerator(bind.accelerator)}
                </span>

                <div className="flex shrink-0 gap-1">
                  <button
                    type="button"
                    onClick={() => run(bind)}
                    className="rounded-md px-2.5 py-1.5 text-xs text-neutral-400 hover:bg-neutral-800 hover:text-neutral-100"
                  >
                    Run
                  </button>
                  <button
                    type="button"
                    onClick={() => setEditing(bind)}
                    className="rounded-md px-2.5 py-1.5 text-xs text-neutral-400 hover:bg-neutral-800 hover:text-neutral-100"
                  >
                    Edit
                  </button>
                  <button
                    type="button"
                    onClick={() => remove(bind)}
                    className="rounded-md px-2.5 py-1.5 text-xs text-neutral-400 hover:bg-neutral-800 hover:text-red-300"
                  >
                    Delete
                  </button>
                </div>
              </li>
            ))}
          </ul>
        )}
      </main>

      {editing ? (
        <Editor
          bind={editing}
          error={error}
          onSave={save}
          onCancel={() => {
            setEditing(null);
            setError("");
          }}
        />
      ) : null}
    </div>
  );
}
