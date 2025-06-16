use reqwest::header::{HeaderMap, RANGE};
use reqwest::Client;
use std::sync::Arc;
use tokio::sync::OnceCell;

use crate::barretenberg::srs::Srs;

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
        // Pobierz zarówno G1 jak i G2
        let (g1_data, g2_data) =
            tokio::try_join!(self.download_g1_data(), self.download_g2_data())?;

        Ok(Srs {
            num_points: self.num_points,
            g1_data,
            g2_data,
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

    async fn download_g2_data(&self) -> Result<Vec<u8>, Box<dyn std::error::Error + Send + Sync>> {
        let client = Client::new();
        let response = client
            .get("https://crs.aztec.network/g2.dat")
            .send()
            .await?;

        let bytes = response.bytes().await?;
        Ok(bytes.to_vec())
    }

    // Metody pomocnicze do pobrania strumieni (jak w TS)
    pub async fn stream_g1_data(
        &self,
    ) -> Result<reqwest::Response, Box<dyn std::error::Error + Send + Sync>> {
        if self.num_points == 0 {
            return Err("Cannot stream G1 data with 0 points".into());
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

        Ok(response)
    }

    pub async fn stream_g2_data(
        &self,
    ) -> Result<reqwest::Response, Box<dyn std::error::Error + Send + Sync>> {
        let client = Client::new();
        let response = client
            .get("https://crs.aztec.network/g2.dat")
            .send()
            .await?;

        Ok(response)
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

// Struktura dla Grumpkin CRS (dodatkowa, jeśli potrzebujesz)
#[derive(Debug, Clone)]
pub struct NetGrumpkinSrs {
    num_points: u32,
    g1_data: Arc<OnceCell<Vec<u8>>>,
}

impl NetGrumpkinSrs {
    pub fn new(num_points: u32) -> Self {
        NetGrumpkinSrs {
            num_points,
            g1_data: Arc::new(OnceCell::new()),
        }
    }

    pub async fn get_g1_data(&self) -> Result<&Vec<u8>, Box<dyn std::error::Error + Send + Sync>> {
        self.g1_data
            .get_or_try_init(|| async { self.download_g1_data().await })
            .await
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
            .get("https://crs.aztec.network/grumpkin_g1.dat")
            .headers(headers)
            .send()
            .await?;

        let bytes = response.bytes().await?;
        Ok(bytes.to_vec())
    }

    pub async fn stream_g1_data(
        &self,
    ) -> Result<reqwest::Response, Box<dyn std::error::Error + Send + Sync>> {
        if self.num_points == 0 {
            return Err("Cannot stream G1 data with 0 points".into());
        }

        let g1_end = self.num_points * 64 - 1;
        let mut headers = HeaderMap::new();
        headers.insert(RANGE, format!("bytes=0-{}", g1_end).parse()?);

        let client = Client::new();
        let response = client
            .get("https://crs.aztec.network/grumpkin_g1.dat")
            .headers(headers)
            .send()
            .await?;

        Ok(response)
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
        assert!(!srs_ref.g1_data.is_empty());
        assert!(!srs_ref.g2_data.is_empty());
    }

    #[tokio::test]
    async fn test_g2_download() {
        let net_srs = NetSrs::new(100);
        let srs = net_srs.get_srs().await.unwrap();
        // G2 powinno być pobrane z sieci, nie z hardkodowanej stałej
        assert!(!srs.g2_data.is_empty());
        println!("G2 data size: {} bytes", srs.g2_data.len());
    }

    #[tokio::test]
    async fn test_grumpkin_srs() {
        let grumpkin_srs = NetGrumpkinSrs::new(500);
        let g1_data = grumpkin_srs.get_g1_data().await.unwrap();
        assert_eq!(g1_data.len(), 500 * 64);
    }
}
