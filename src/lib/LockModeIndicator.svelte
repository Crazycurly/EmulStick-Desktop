<script lang="ts">
  import { invoke } from "@tauri-apps/api/core";
  import { listen } from "@tauri-apps/api/event";
  import { onMount } from "svelte";
  import { lockMode, controlActive } from "./stores";

  let locked = $state(false);
  let active = $state(false);
  let initialized = $state(false);

  lockMode.subscribe((v) => (locked = v));
  controlActive.subscribe((v) => (active = v));

  onMount(async () => {
    if (!initialized) {
      initialized = true;
      // Load initial states from backend
      try {
        const [lockModeVal, controlActiveVal] = await Promise.all([
          invoke<boolean>("get_lock_mode"),
          invoke<boolean>("is_control_active"),
        ]);
        lockMode.set(lockModeVal);
        controlActive.set(controlActiveVal);
      } catch (e) {
        console.error("Failed to load initial state:", e);
      }
    }

    const unlisten = listen<boolean>("lock-mode-changed", (event) => {
      lockMode.set(event.payload);
    });
    return () => {
      unlisten.then((fn) => fn());
    };
  });

  async function toggleLock() {
    try {
      const newMode = await invoke<boolean>("toggle_lock_mode");
      lockMode.set(newMode);
    } catch (e) {
      console.error("Toggle lock failed:", e);
    }
  }

  async function toggleControl() {
    try {
      if (active) {
        await invoke("stop_control");
        controlActive.set(false);
      } else {
        await invoke("start_control");
        controlActive.set(true);
      }
    } catch (e) {
      console.error("Toggle control failed:", e);
    }
  }
</script>

<div class="lock-controls">
  <button
    class="control-btn"
    class:active
    onclick={toggleControl}
  >
    {active ? "Stop Control" : "Start Control"}
  </button>

  <button
    class="lock-btn"
    class:locked
    onclick={toggleLock}
    disabled={!active}
  >
    {locked ? "🔒 LOCKED" : "🔓 Unlocked"}
  </button>

  {#if active}
    <span class="hint">Press R-Ctrl + R-Alt to toggle lock</span>
  {/if}
</div>

{#if locked}
  <div class="lock-overlay">
    <span>🔒 Control Locked — All input forwarded to remote</span>
  </div>
{/if}

<style>
  .lock-controls {
    display: flex;
    align-items: center;
    gap: 0.75rem;
    padding: 0.5rem 1rem;
  }
  .control-btn {
    padding: 0.4rem 1rem;
    background: #22c55e;
    color: white;
    border: none;
    border-radius: 6px;
    cursor: pointer;
    font-size: 0.85rem;
    font-weight: 600;
  }
  .control-btn.active {
    background: #ef4444;
  }
  .lock-btn {
    padding: 0.4rem 1rem;
    background: transparent;
    border: 1px solid rgba(255, 255, 255, 0.2);
    color: inherit;
    border-radius: 6px;
    cursor: pointer;
    font-size: 0.85rem;
  }
  .lock-btn:disabled {
    opacity: 0.4;
    cursor: not-allowed;
  }
  .lock-btn.locked {
    background: rgba(239, 68, 68, 0.2);
    border-color: #ef4444;
    color: #ef4444;
  }
  .hint {
    font-size: 0.75rem;
    opacity: 0.5;
  }
  .lock-overlay {
    position: fixed;
    top: 0;
    left: 0;
    right: 0;
    background: rgba(239, 68, 68, 0.9);
    color: white;
    text-align: center;
    padding: 0.4rem;
    font-size: 0.85rem;
    font-weight: 600;
    z-index: 1000;
  }
</style>