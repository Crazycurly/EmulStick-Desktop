use serde::Serialize;

use btleplug::api::{
    Central, Characteristic, Manager as _, Peripheral as _, ScanFilter, WriteType,
};
use btleplug::platform::{Adapter, Manager, Peripheral};
use tokio::sync::Mutex;
use uuid::Uuid;

/// Custom service UUID (F800)
#[allow(dead_code)]
const SERVICE_UUID: Uuid = Uuid::from_u128(0x0000F800_0000_1000_8000_00805f9b34fb);
/// F801 — Keyboard (write + notify)
const KEYBOARD_CHAR_UUID: Uuid = Uuid::from_u128(0x0000F801_0000_1000_8000_00805f9b34fb);
/// F802 — Gamepad (write only)
const GAMEPAD_CHAR_UUID: Uuid = Uuid::from_u128(0x0000F802_0000_1000_8000_00805f9b34fb);
/// F803 — Mouse (write only)
const MOUSE_CHAR_UUID: Uuid = Uuid::from_u128(0x0000F803_0000_1000_8000_00805f9b34fb);
/// F804 — Pen & Consumer (write only)
const CONSUMER_CHAR_UUID: Uuid = Uuid::from_u128(0x0000F804_0000_1000_8000_00805f9b34fb);
/// F80F — Private Control (write + notify — reserved, do not use)
#[allow(dead_code)]
const PRIVATE_CHAR_UUID: Uuid = Uuid::from_u128(0x0000F80F_0000_1000_8000_00805f9b34fb);

#[derive(Debug, Clone, Serialize)]
pub struct BleDeviceInfo {
    pub name: String,
    pub address: String,
    pub rssi: Option<i16>,
}

struct ConnectedDevice {
    peripheral: Peripheral,
    keyboard_char: Characteristic,
    gamepad_char: Characteristic,
    mouse_char: Characteristic,
    consumer_char: Characteristic,
}

pub struct BleState {
    manager: Manager,
    connected: Mutex<Option<ConnectedDevice>>,
}

impl BleState {
    pub async fn new() -> Result<Self, String> {
        let manager = Manager::new()
            .await
            .map_err(|e| format!("Failed to create BLE manager: {}", e))?;
        Ok(Self {
            manager,
            connected: Mutex::new(None),
        })
    }

    async fn get_adapter(&self) -> Result<Adapter, String> {
        let adapters = self
            .manager
            .adapters()
            .await
            .map_err(|e| format!("Failed to get adapters: {}", e))?;
        adapters
            .into_iter()
            .next()
            .ok_or_else(|| "No Bluetooth adapter found".to_string())
    }

    pub async fn scan_devices(&self) -> Result<Vec<BleDeviceInfo>, String> {
        let adapter = self.get_adapter().await?;

        adapter
            .start_scan(ScanFilter::default())
            .await
            .map_err(|e| format!("Failed to start scan: {}", e))?;

        // Allow time for scan results
        tokio::time::sleep(std::time::Duration::from_secs(3)).await;

        adapter
            .stop_scan()
            .await
            .map_err(|e| format!("Failed to stop scan: {}", e))?;

        let peripherals = adapter
            .peripherals()
            .await
            .map_err(|e| format!("Failed to get peripherals: {}", e))?;

        let mut devices = Vec::new();
        for p in peripherals {
            if let Ok(Some(props)) = p.properties().await {
                let name = props.local_name.unwrap_or_default();
                if !name.is_empty() {
                    devices.push(BleDeviceInfo {
                        name,
                        address: props.address.to_string(),
                        rssi: props.rssi,
                    });
                }
            }
        }

        Ok(devices)
    }

    pub async fn connect(&self, address: &str) -> Result<(), String> {
        let adapter = self.get_adapter().await?;

        // Re-scan briefly so the adapter rediscovers peripherals
        adapter
            .start_scan(ScanFilter::default())
            .await
            .map_err(|e| format!("Failed to start scan: {}", e))?;
        tokio::time::sleep(std::time::Duration::from_secs(2)).await;
        adapter
            .stop_scan()
            .await
            .map_err(|e| format!("Failed to stop scan: {}", e))?;

        let peripherals = adapter
            .peripherals()
            .await
            .map_err(|e| format!("Failed to get peripherals: {}", e))?;

        let mut target = None;
        for p in peripherals {
            if let Ok(Some(props)) = p.properties().await {
                if props.address.to_string() == address {
                    target = Some(p);
                    break;
                }
            }
        }

        let peripheral = target.ok_or_else(|| format!("Device {} not found", address))?;

        peripheral
            .connect()
            .await
            .map_err(|e| format!("Failed to connect: {}", e))?;

        peripheral
            .discover_services()
            .await
            .map_err(|e| format!("Failed to discover services: {}", e))?;

        let chars = peripheral.characteristics();

        let keyboard_char = chars
            .iter()
            .find(|c| c.uuid == KEYBOARD_CHAR_UUID)
            .cloned()
            .ok_or_else(|| "Keyboard characteristic not found".to_string())?;

        let gamepad_char = chars
            .iter()
            .find(|c| c.uuid == GAMEPAD_CHAR_UUID)
            .cloned()
            .ok_or_else(|| "Gamepad characteristic not found".to_string())?;

        let mouse_char = chars
            .iter()
            .find(|c| c.uuid == MOUSE_CHAR_UUID)
            .cloned()
            .ok_or_else(|| "Mouse characteristic not found".to_string())?;

        let consumer_char = chars
            .iter()
            .find(|c| c.uuid == CONSUMER_CHAR_UUID)
            .cloned()
            .ok_or_else(|| "Consumer characteristic not found".to_string())?;

        let mut connected = self.connected.lock().await;
        *connected = Some(ConnectedDevice {
            peripheral,
            keyboard_char,
            gamepad_char,
            mouse_char,
            consumer_char,
        });

        Ok(())
    }

    pub async fn disconnect(&self) -> Result<(), String> {
        let mut connected = self.connected.lock().await;
        if let Some(device) = connected.take() {
            device
                .peripheral
                .disconnect()
                .await
                .map_err(|e| format!("Failed to disconnect: {}", e))?;
        }
        Ok(())
    }

    pub async fn is_connected(&self) -> bool {
        let connected = self.connected.lock().await;
        if let Some(device) = connected.as_ref() {
            device.peripheral.is_connected().await.unwrap_or(false)
        } else {
            false
        }
    }

    pub async fn write_keyboard(&self, payload: [u8; 8]) -> Result<(), String> {
        let connected = self.connected.lock().await;
        let device = connected
            .as_ref()
            .ok_or_else(|| "Not connected".to_string())?;

        device
            .peripheral
            .write(&device.keyboard_char, &payload, WriteType::WithoutResponse)
            .await
            .map_err(|e| format!("Failed to write keyboard: {}", e))
    }

    pub async fn write_mouse(&self, payload: [u8; 6]) -> Result<(), String> {
        let connected = self.connected.lock().await;
        let device = connected
            .as_ref()
            .ok_or_else(|| "Not connected".to_string())?;

        device
            .peripheral
            .write(&device.mouse_char, &payload, WriteType::WithoutResponse)
            .await
            .map_err(|e| format!("Failed to write mouse: {}", e))
    }

    /// Send a 10-byte gamepad report to characteristic F802.
    pub async fn write_gamepad(&self, payload: [u8; 10]) -> Result<(), String> {
        let connected = self.connected.lock().await;
        let device = connected
            .as_ref()
            .ok_or_else(|| "Not connected".to_string())?;

        device
            .peripheral
            .write(&device.gamepad_char, &payload, WriteType::WithoutResponse)
            .await
            .map_err(|e| format!("Failed to write gamepad: {}", e))
    }

    /// Send a pen (6-byte) or consumer (4-byte) report to characteristic F804.
    pub async fn write_consumer(&self, payload: &[u8]) -> Result<(), String> {
        let connected = self.connected.lock().await;
        let device = connected
            .as_ref()
            .ok_or_else(|| "Not connected".to_string())?;

        device
            .peripheral
            .write(&device.consumer_char, payload, WriteType::WithoutResponse)
            .await
            .map_err(|e| format!("Failed to write consumer: {}", e))
    }
}
