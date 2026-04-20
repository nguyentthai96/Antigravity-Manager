use serde::{Deserialize, Serialize};

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct DeviceProfile {
    #[serde(rename = "machineId")]
    pub machine_id: String,
    #[serde(rename = "macMachineId")]
    pub mac_machine_id: String,
    #[serde(rename = "devDeviceId")]
    pub dev_device_id: String,
    #[serde(rename = "sqmId")]
    pub sqm_id: String,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct AccountItem {
    pub email: String,
    pub refresh_token: String,
}
