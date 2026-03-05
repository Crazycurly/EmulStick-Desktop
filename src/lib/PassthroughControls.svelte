<script lang="ts">
  import { invoke } from "@tauri-apps/api/core";
  import { passthroughConfig } from "./stores";

  let keyboard = $state(true);
  let mouse = $state(true);
  let video = $state(true);

  passthroughConfig.subscribe((c) => {
    keyboard = c.keyboard;
    mouse = c.mouse;
    video = c.video;
  });

  async function update(field: "keyboard" | "mouse" | "video", value: boolean) {
    const next = { keyboard, mouse, video, [field]: value };
    passthroughConfig.set(next);
    try {
      await invoke("set_passthrough", {
        enableKeyboard: next.keyboard,
        enableMouse: next.mouse,
        enableVideo: next.video,
      });
    } catch (e) {
      console.error("Failed to update passthrough:", e);
    }
  }
</script>

<div class="controls">
  <h3>Passthrough</h3>
  <label class="toggle">
    <input
      type="checkbox"
      checked={keyboard}
      onchange={(e) => update("keyboard", (e.target as HTMLInputElement).checked)}
    />
    <span class="toggle-label">&#9000; Keyboard</span>
  </label>
  <label class="toggle">
    <input
      type="checkbox"
      checked={mouse}
      onchange={(e) => update("mouse", (e.target as HTMLInputElement).checked)}
    />
    <span class="toggle-label">&#128433; Mouse</span>
  </label>
  <label class="toggle">
    <input
      type="checkbox"
      checked={video}
      onchange={(e) => update("video", (e.target as HTMLInputElement).checked)}
    />
    <span class="toggle-label">&#128250; Video</span>
  </label>
</div>

<style>
  .controls {
    padding: 1rem;
  }
  .controls h3 {
    margin: 0 0 0.75rem 0;
    font-size: 0.9rem;
    text-transform: uppercase;
    letter-spacing: 0.05em;
    opacity: 0.7;
  }
  .toggle {
    display: flex;
    align-items: center;
    gap: 0.6rem;
    padding: 0.5rem 0;
    cursor: pointer;
  }
  .toggle input[type="checkbox"] {
    width: 36px;
    height: 20px;
    appearance: none;
    background: #444;
    border-radius: 10px;
    position: relative;
    cursor: pointer;
    transition: background 0.2s;
    flex-shrink: 0;
  }
  .toggle input[type="checkbox"]::before {
    content: "";
    position: absolute;
    top: 2px;
    left: 2px;
    width: 16px;
    height: 16px;
    background: white;
    border-radius: 50%;
    transition: transform 0.2s;
  }
  .toggle input[type="checkbox"]:checked {
    background: #3b82f6;
  }
  .toggle input[type="checkbox"]:checked::before {
    transform: translateX(16px);
  }
  .toggle-label {
    font-size: 0.9rem;
  }
</style>
