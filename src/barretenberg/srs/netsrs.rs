use reqwest::header::{HeaderMap, RANGE};
use reqwest::Client;
use std::ops::Deref;
use std::sync::Arc;
use tokio::sync::OnceCell;

use super::{Srs, G2};

#[derive(Debug, Clone)]
pub struct NetSrs {
    num_points: u32,
    srs: Arc<OnceCell<Srs>>,
}

// Implementacja Clone dla Srs - tylko jeśli nie ma derive w originalnej definicji
impl Clone for Srs {
    fn clone(&self) -> Self {
        Srs {
            num_points: self.num_points,
            g1_data: self.g1_data.clone(),
            g2_data: self.g2_data.clone(),
        }
    }
}

impl NetSrs {
    pub fn new(num_points: u32) -> Self {
        NetSrs {
            num_points,
            srs: Arc::new(OnceCell::new()),
        }
    }

    pub async fn get_srs(&self) -> Result<&Srs, Box<dyn std::error::Error + Send + Sync>> {
        self.srs
            .get_or_try_init(|| async { self.download_srs().await })
            .await
    }

    async fn download_srs(&self) -> Result<Srs, Box<dyn std::error::Error + Send + Sync>> {
        Ok(Srs {
            num_points: self.num_points,
            g1_data: self.download_g1_data().await?,
            g2_data: G2.to_vec(),
        })
    }

    async fn download_g1_data(&self) -> Result<Vec<u8>, Box<dyn std::error::Error + Send + Sync>> {
        if self.num_points == 0 {
            return Ok(Vec::new());
        }

        let g1_end = self.num_points * 64 - 1;

        let mut headers = HeaderMap::new();
        headers.insert(RANGE, format!("bytes=0-{}", g1_end).parse()?);

        let client = Client::new();
        let response = client
            .get("https://crs.aztec.network/g1.dat")
            .headers(headers)
            .send()
            .await?;

        let bytes = response.bytes().await?;
        Ok(bytes.to_vec())
    }

    // Metoda która próbuje sklonować SRS jeśli jest dostępny
    pub fn to_srs(self) -> Result<Srs, Box<dyn std::error::Error + Send + Sync>> {
        self.srs
            .get()
            .cloned()
            .ok_or("SRS not initialized. Call get_srs() first.".into())
    }

    // Alternatywna metoda która move'uje wartość bez klonowania
    pub fn try_to_srs(self) -> Result<Srs, Box<dyn std::error::Error + Send + Sync>> {
        match Arc::try_unwrap(self.srs) {
            Ok(once_cell) => match once_cell.into_inner() {
                Some(srs) => Ok(srs),
                None => Err("SRS not initialized. Call get_srs() first.".into()),
            },
            Err(_) => Err("Cannot extract SRS: multiple references exist".into()),
        }
    }

    // Metoda która zwraca referencję do SRS
    pub async fn as_srs(&self) -> Result<&Srs, Box<dyn std::error::Error + Send + Sync>> {
        self.get_srs().await
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn test_lazy_srs() {
        let net_srs = NetSrs::new(1000);
        let srs_ref = net_srs.get_srs().await.unwrap();
        assert_eq!(srs_ref.num_points, 1000);

        // Teraz try_to_srs będzie dostępne
        let srs_owned = net_srs.try_to_srs().unwrap();
        assert_eq!(srs_owned.num_points, 1000);
    }

    #[tokio::test]
    async fn test_lazy_srs_reference() {
        let net_srs = NetSrs::new(1000);
        let srs_ref = net_srs.as_srs().await.unwrap();
        assert_eq!(srs_ref.num_points, 1000);
        // net_srs może być dalej używany
    }

    #[tokio::test]
    async fn test_to_srs_with_clone() {
        let net_srs = NetSrs::new(1000);
        // Najpierw zainicjuj SRS
        let _ = net_srs.get_srs().await.unwrap();

        // Teraz użyj to_srs (które klonuje)
        let srs_owned = net_srs.to_srs().unwrap();
        assert_eq!(srs_owned.num_points, 1000);
    }
}
