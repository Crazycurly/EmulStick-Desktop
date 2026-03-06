<script lang="ts">
  import { invoke } from "@tauri-apps/api/core";
  import { connectionState } from "./stores";
  import { onMount, onDestroy } from "svelte";

  let connected = $state(false);
  let deviceName = $state("");
  let address = $state("");
  let checkInterval: ReturnType<typeof setInterval> | null = null;

  connectionState.subscribe((s) => {
    connected = s.connected;
    deviceName = s.deviceName;
    address = s.address;
  });

  async function checkConnection() {
    try {
      const isConnected = await invoke<boolean>("get_connection_status");
      if (isConnected !== connected) {
        if (!isConnected) {
          // Device disconnected externally
          connectionState.set({ connected: false, deviceName: "", address: "" });
        }
      }
    } catch (e) {
      console.error("Connection check failed:", e);
    }
  }

  onMount(() => {
    // Check connection status every 3 seconds
    checkInterval = setInterval(checkConnection, 3000);
  });

  onDestroy(() => {
    if (checkInterval) {
      clearInterval(checkInterval);
    }
  });

  async function disconnect() {
    try {
      await invoke("disconnect_ble_device");
      connectionState.set({ connected: false, deviceName: "", address: "" });
    } catch (e) {
      console.error("Disconnect failed:", e);
    }
  }
</script>

<div class="status" class:connected>
  <div class="indicator"></div>
  <div class="info">
    {#if connected}
      <span class="label">{deviceName || "Connected"}</span>
      <span class="addr">{address}</span>
    {:else}
      <span class="label">Disconnected</span>
    {/if}
  </div>
  {#if connected}
    <button class="disconnect-btn" onclick={disconnect}>Disconnect</button>
  {/if}
</div>

<style>
  .status {
    display: flex;
    align-items: center;
    gap: 0.6rem;
    padding: 0.5rem 1rem;
    background: rgba(255, 255, 255, 0.03);
    border-radius: 8px;
  }
  .indicator {
    width: 10px;
    height: 10px;
    border-radius: 50%;
    background: #ef4444;
    flex-shrink: 0;
  }
  .connected .indicator {
    background: #22c55e;
    box-shadow: 0 0 8px rgba(34, 197, 94, 0.5);
  }
  .info {
    display: flex;
    flex-direction: column;
    flex: 1;
    min-width: 0;
  }
  .label {
    font-weight: 600;
    font-size: 0.9rem;
  }
  .addr {
    font-size: 0.75rem;
    opacity: 0.5;
    overflow: hidden;
    text-overflow: ellipsis;
    white-space: nowrap;
  }
  .disconnect-btn {
    padding: 0.3rem 0.6rem;
    background: transparent;
    border: 1px solid rgba(239, 68, 68, 0.5);
    color: #ef4444;
    border-radius: 4px;
    cursor: pointer;
    font-size: 0.75rem;
    flex-shrink: 0;
  }
  .disconnect-btn:hover {
    background: rgba(239, 68, 68, 0.1);
  }
</style>