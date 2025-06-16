pub mod localsrs;
pub mod netsrs;
use serde::{Deserialize, Serialize};

use crate::{
    barretenberg::utils::{compute_subgroup_size, get_circuit_size},
    init_slab_allocator_safe, srs_init_safe,
};

// G2 is a small fixed group, so we can hardcode it here
const G2: [u8; 128] = [
    126, 35, 31, 236, 147, 136, 131, 176, 159, 89, 68, 7, 59, 50, 7, 139, 188, 137, 181, 179, 152,
    181, 151, 78, 1, 24, 196, 213, 184, 55, 188, 194, 78, 254, 48, 250, 192, 147, 131, 193, 234,
    81, 216, 122, 53, 142, 3, 139, 231, 255, 78, 88, 7, 145, 222, 232, 38, 14, 1, 178, 81, 246,
    241, 199, 133, 74, 135, 212, 218, 204, 94, 85, 17, 230, 221, 63, 150, 230, 206, 162, 86, 71,
    91, 66, 20, 229, 97, 94, 34, 254, 189, 163, 192, 192, 99, 42, 238, 65, 60, 128, 218, 106, 95,
    228, 156, 242, 160, 70, 65, 249, 155, 164, 210, 81, 86, 193, 187, 154, 114, 133, 4, 252, 99,
    105, 247, 17, 15, 227,
];

#[derive(Serialize, Deserialize, PartialEq, Debug)]
pub struct Srs {
    pub g1_data: Vec<u8>,
    pub g2_data: Vec<u8>,
    pub num_points: u32,
}

impl Srs {
    pub fn get(self, num_points: u32) -> Srs {
        match self.num_points.cmp(&num_points) {
            std::cmp::Ordering::Equal => self,
            _ => Srs {
                g1_data: self.g1_data[..=(num_points * 64 - 1) as usize].to_vec(),
                g2_data: self.g2_data,
                num_points: num_points,
            },
        }
    }
}

pub async fn get_srs(
    subgroup_size: u32,
    srs_path: Option<&str>,
) -> Result<Srs, Box<dyn std::error::Error + Send + Sync>> {
    match srs_path {
        Some(path) => {
            if path.ends_with(".dat") {
                // Interpret as a .dat file
                let local_srs = localsrs::LocalSrs::from_dat_file(subgroup_size + 1, Some(path));
                Ok(local_srs.to_srs())
            } else {
                // Otherwise interpret as a .local file (i.e. a serialized SRS struct)
                let local_srs = localsrs::LocalSrs::new(subgroup_size + 1, Some(path));
                Ok(local_srs.to_srs())
            }
        }
        None => {
            eprintln!("IN NET SRS");
            let net_srs = netsrs::NetSrs::new(subgroup_size + 1);
            eprintln!("{:?}", net_srs);

            // Pobierz SRS async i następnie wyciągnij owned wartość
            let _ = net_srs.get_srs().await?;
            net_srs.try_to_srs()
        }
    }
}

pub async fn setup_srs(circuit_size: u32, srs_path: Option<&str>) -> Result<u32, String> {
    eprintln!("=== SRS Setup Debug ===");
    eprintln!("Circuit size: {}", circuit_size);

    // 1) Calculate subgroup size
    let subgroup_size = compute_subgroup_size(circuit_size);
    eprintln!("Subgroup size: {}", subgroup_size);

    // 2) Get SRS data (await the async function)
    eprintln!("Getting SRS data...");
    let srs = get_srs(subgroup_size, srs_path)
        .await
        .map_err(|e| format!("Failed to get SRS: {}", e))?;

    // 3) Validate data
    eprintln!("  G1 data length: {} bytes", srs.g1_data.len());
    eprintln!("  G2 data length: {} bytes", srs.g2_data.len());
    eprintln!("  Num points: {}", srs.num_points);
    srs_init_safe(&srs.g1_data, srs.num_points, &srs.g2_data);

    eprintln!("SRS initialized successfully!");
    Ok(srs.num_points)
}

pub async fn setup_srs_from_bytecode(
    circuit_bytecode: &str,
    srs_path: Option<&str>,
    recursive: bool,
) -> Result<u32, String> {
    let circuit_size = get_circuit_size(circuit_bytecode, recursive);
    setup_srs(circuit_size, srs_path).await
}
