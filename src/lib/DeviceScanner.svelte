<script lang="ts">
  import { invoke } from "@tauri-apps/api/core";
  import { connectionState } from "./stores";

  interface BleDevice {
    name: string;
    address: string;
    rssi: number | null;
  }

  let devices = $state<BleDevice[]>([]);
  let scanning = $state(false);
  let error = $state("");

  async function scan() {
    scanning = true;
    error = "";
    try {
      devices = await invoke<BleDevice[]>("scan_ble_devices");
    } catch (e) {
      error = String(e);
    } finally {
      scanning = false;
    }
  }

  async function connectDevice(device: BleDevice) {
    error = "";
    try {
      await invoke("connect_ble_device", { address: device.address });
      connectionState.set({
        connected: true,
        deviceName: device.name,
        address: device.address,
      });
    } catch (e) {
      error = String(e);
    }
  }
</script>

<div class="scanner">
  <div class="scanner-header">
    <h3>BLE Devices</h3>
    <button onclick={scan} disabled={scanning}>
      {scanning ? "Scanning..." : "Scan"}
    </button>
  </div>

  {#if error}
    <p class="error">{error}</p>
  {/if}

  <div class="device-list">
    {#each devices as device}
      <button class="device-item" onclick={() => connectDevice(device)}>
        <span class="device-name">{device.name}</span>
        <span class="device-info">
          {device.address}
          {#if device.rssi !== null}
            <span class="rssi">{device.rssi} dBm</span>
          {/if}
        </span>
      </button>
    {:else}
      {#if !scanning}
        <p class="empty">No devices found. Click Scan to search.</p>
      {/if}
    {/each}
  </div>
</div>

<style>
  .scanner {
    padding: 1rem;
  }
  .scanner-header {
    display: flex;
    justify-content: space-between;
    align-items: center;
    margin-bottom: 0.75rem;
  }
  .scanner-header h3 {
    margin: 0;
    font-size: 0.9rem;
    text-transform: uppercase;
    letter-spacing: 0.05em;
    opacity: 0.7;
  }
  .scanner-header button {
    padding: 0.4rem 1rem;
    background: #3b82f6;
    color: white;
    border: none;
    border-radius: 6px;
    cursor: pointer;
    font-size: 0.85rem;
  }
  .scanner-header button:disabled {
    opacity: 0.5;
    cursor: not-allowed;
  }
  .device-list {
    display: flex;
    flex-direction: column;
    gap: 0.5rem;
  }
  .device-item {
    display: flex;
    flex-direction: column;
    align-items: flex-start;
    padding: 0.75rem;
    background: rgba(255, 255, 255, 0.05);
    border: 1px solid rgba(255, 255, 255, 0.1);
    border-radius: 8px;
    cursor: pointer;
    color: inherit;
    text-align: left;
    width: 100%;
  }
  .device-item:hover {
    background: rgba(255, 255, 255, 0.1);
    border-color: #3b82f6;
  }
  .device-name {
    font-weight: 600;
    font-size: 0.95rem;
  }
  .device-info {
    font-size: 0.8rem;
    opacity: 0.6;
    margin-top: 0.2rem;
  }
  .rssi {
    margin-left: 0.5rem;
  }
  .error {
    color: #ef4444;
    font-size: 0.85rem;
    margin: 0.5rem 0;
  }
  .empty {
    opacity: 0.5;
    font-size: 0.85rem;
    text-align: center;
    padding: 1rem;
  }
</style>
