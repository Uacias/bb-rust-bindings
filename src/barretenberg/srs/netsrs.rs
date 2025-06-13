use reqwest::blocking::Client;
use reqwest::header::{HeaderMap, RANGE};
use std::ops::Deref;

use super::{Srs, G2};

#[derive(Debug)]
pub struct NetSrs(pub Srs);

impl Deref for NetSrs {
    type Target = Srs;
    fn deref(&self) -> &Self::Target {
        &self.0
    }
}

impl NetSrs {
    pub fn new(num_points: u32) -> Self {
        NetSrs(Srs {
            num_points,
            g1_data: Self::download_g1_data(num_points),
            g2_data: G2.to_vec(),
        })
    }

    pub fn to_srs(self) -> Srs {
        self.0
    }

    fn download_g1_data(num_points: u32) -> Vec<u8> {
        // Skip download if numPoints is 0
        if num_points == 0 {
            return Vec::new();
        }

        let g1_end = num_points * 64 - 1;

        let mut headers = HeaderMap::new();
        headers.insert(RANGE, format!("bytes=0-{}", g1_end).parse().unwrap());

        let response = Client::new()
            .get("https://crs.aztec.network/g1.dat")
            .headers(headers)
            .send()
            .unwrap();

        response.bytes().unwrap().to_vec()
    }
}
