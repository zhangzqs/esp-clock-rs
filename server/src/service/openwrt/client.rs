use anyhow::Result;
use reqwest::redirect::Policy;
use serde::Deserialize;

pub struct Client {
    username: String,
    password: String,
    luci_addr: String,
    client: reqwest::Client,
}

#[derive(Debug, Deserialize)]
pub struct ClientConfig {
    username: String,
    password: String,
    max_redirects: usize,
    luci_addr: String,
}

impl Client {
    pub fn new(config: ClientConfig) -> Self {
        Client {
            username: config.username,
            password: config.password,
            luci_addr: config.luci_addr,
            client: reqwest::Client::builder()
                .redirect(Policy::limited(config.max_redirects))
                .cookie_store(true)
                .build()
                .unwrap(),
        }
    }

    pub async fn login(&self) -> Result<()> {
        let url = format!("{}/cgi-bin/luci", self.luci_addr);
        let form = [
            ("luci_username", &self.username),
            ("luci_password", &self.password),
        ];
        let res = self.client.post(url).form(&form).send().await.unwrap();
        if res.status() != 200 {
            panic!("login failed");
        }
        Ok(())
    }
}

#[derive(Debug)]
pub struct ClientNetworkInfo {
    pub device_name: String,
    pub mac_address: String,
    pub upload_rate: u64,
    pub download_rate: u64,
    pub total_download: u64,
    pub total_upload: u64,
    pub unknown_field_1: String,
    pub unknown_field_2: String,
    pub unknown_field_3: String,
    pub ip: String,
}

#[derive(Debug)]
pub struct TotalNetworkInfo {
    pub total_download_rate: u64,
    pub total_upload_rate: u64,
    pub total_download: u64,
    pub total_upload: u64,
    pub total: u64,
}

#[derive(Debug)]
pub struct AllClientNetworkInfo {
    pub clients: Vec<ClientNetworkInfo>,
    pub total: TotalNetworkInfo,
}

impl Client {
    pub async fn get_all_client_network_info(&self) -> AllClientNetworkInfo {
        #[derive(Debug, Deserialize)]
        #[serde(untagged)]
        enum ResponseData {
            ClientsInfo(Vec<Vec<serde_json::Value>>),
            TotalInfo(Vec<usize>),
        }

        let raw_data = self
            .client
            .get(format!("{}/cgi-bin/luci/admin/nlbw/usage", self.luci_addr))
            .query(&[("proto", "ipv4")])
            .send()
            .await
            .unwrap()
            .json::<Vec<ResponseData>>()
            .await
            .unwrap();

        let clients_info = if let ResponseData::ClientsInfo(ref x) = raw_data[0] {
            x.iter()
                .map(|x| {
                    let mut d = x.iter();
                    // device name, mac addr, download rate, upload rate, total download, total upload, *, *, *, ip
                    ClientNetworkInfo {
                        device_name: d.next().unwrap().as_str().unwrap().to_string(),
                        mac_address: d.next().unwrap().as_str().unwrap().to_string(),
                        upload_rate: d.next().unwrap().as_str().unwrap().parse().unwrap(),
                        download_rate: d.next().unwrap().as_str().unwrap().parse().unwrap(),
                        total_download: d.next().unwrap().as_str().unwrap().parse().unwrap(),
                        total_upload: d.next().unwrap().as_str().unwrap().parse().unwrap(),
                        unknown_field_1: d.next().unwrap().as_str().unwrap().to_string(),
                        unknown_field_2: d.next().unwrap().as_str().unwrap().to_string(),
                        unknown_field_3: d.next().unwrap().as_str().unwrap().to_string(),
                        ip: d.next().unwrap().as_str().unwrap().to_string(),
                    }
                })
                .collect::<Vec<_>>()
        } else {
            panic!("error")
        };
        let total = if let ResponseData::TotalInfo(ref x) = raw_data[1] {
            // total download rate, total upload rate, total download, total upload, total
            TotalNetworkInfo {
                total_download_rate: x[0] as u64,
                total_upload_rate: x[1] as u64,
                total_download: x[2] as u64,
                total_upload: x[3] as u64,
                total: x[4] as u64,
            }
        } else {
            panic!("error")
        };

        AllClientNetworkInfo {
            clients: clients_info,
            total,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    #[tokio::test]
    async fn test_a() {
        let cli = Client::new(ClientConfig {
            username: "root".to_string(),
            password: "12345".to_string(),
            max_redirects: 10,
            luci_addr: "http://openwrt.zzq".to_string(),
        });
        cli.login().await.unwrap();
        let info = cli.get_all_client_network_info().await;
        println!("{:#?}", info);
    }
}
