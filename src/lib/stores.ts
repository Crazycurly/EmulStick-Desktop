import { writable } from "svelte/store";

export const connectionState = writable<{
  connected: boolean;
  deviceName: string;
  address: string;
}>({
  connected: false,
  deviceName: "",
  address: "",
});

export const passthroughConfig = writable<{
  keyboard: boolean;
  mouse: boolean;
  video: boolean;
}>({
  keyboard: true,
  mouse: false,
  video: false,
});

export const lockMode = writable<boolean>(false);

export const controlActive = writable<boolean>(false);