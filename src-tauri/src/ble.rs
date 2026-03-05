use serde::Serialize;

use btleplug::api::{
    Central, Characteristic, Manager as _, Peripheral as _, ScanFilter, WriteType,
};
use btleplug::platform::{Adapter, Manager, Peripheral};
use tokio::sync::Mutex;
use uuid::Uuid;

const SERVICE_UUID: Uuid = Uuid::from_u128(0x0000F800_0000_1000_8000_00805f9b34fb);
const KEYBOARD_CHAR_UUID: Uuid = Uuid::from_u128(0x0000F801_0000_1000_8000_00805f9b34fb);
const MOUSE_CHAR_UUID: Uuid = Uuid::from_u128(0x0000F803_0000_1000_8000_00805f9b34fb);

#[derive(Debug, Clone, Serialize)]
pub struct BleDeviceInfo {
    pub name: String,
    pub address: String,
    pub rssi: Option<i16>,
}

struct ConnectedDevice {
    peripheral: Peripheral,
    keyboard_char: Characteristic,
    mouse_char: Characteristic,
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
        let peripherals = adapter
            .peripherals()
            .await
            .map_err(|e| format!("Failed to get peripherals: {}", e))?;

        let target = peripherals
            .into_iter()
            .find(|p| {
                p.properties()
                    .now_or_never()
                    .and_then(|r| r.ok())
                    .flatten()
                    .map(|props| props.address.to_string() == address)
                    .unwrap_or(false)
            });

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

        let mouse_char = chars
            .iter()
            .find(|c| c.uuid == MOUSE_CHAR_UUID)
            .cloned()
            .ok_or_else(|| "Mouse characteristic not found".to_string())?;

        let mut connected = self.connected.lock().await;
        *connected = Some(ConnectedDevice {
            peripheral,
            keyboard_char,
            mouse_char,
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

    pub async fn write_mouse(&self, payload: [u8; 8]) -> Result<(), String> {
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
}

use futures::FutureExt;
