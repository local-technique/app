use chrono::Utc;
use hmac::{Hmac, Mac, KeyInit};
use sha2::{Digest, Sha256};

type HmacSha256 = Hmac<Sha256>;

#[allow(dead_code)]
#[derive(Clone)]
pub struct S3Signer {
    endpoint: String,
    bucket: String,
    access_key: String,
    secret_key: String,
    region: String,
}

impl S3Signer {
    pub fn new(endpoint: String, bucket: String, access_key: String, secret_key: String) -> Self {
        Self {
            endpoint,
            bucket,
            access_key,
            secret_key,
            region: "auto".to_string(),
        }
    }

    pub fn generate_presigned_get_url(&self, storage_key: &str, expires_secs: u64) -> String {
        let resource = format!("{}/{}", self.bucket, storage_key);
        let expires = expires_secs.to_string();
        let now = Utc::now();
        let date_str = now.format("%Y%m%d").to_string();
        let date_time_str = now.format("%Y%m%dT%H%M%SZ").to_string();

        let host = self.host_name();
        let headers = vec![("host", host.as_str())];

        let signed_headers = "host";
        let query_params = format!(
            "X-Amz-Algorithm=AWS4-HMAC-SHA256&X-Amz-Credential={}%2F{}%2F{}%2Fs3%2Faws4_request&X-Amz-Date={}&X-Amz-Expires={}&X-Amz-SignedHeaders={}",
            &self.access_key, &date_str, &self.region, &date_time_str, &expires, signed_headers
        );

        let canonical_request = format!(
            "GET\n/{}\n{}\n{}\n\n{}\nUNSIGNED-PAYLOAD",
            resource, query_params, self._canonical_headers(&headers), signed_headers
        );

        let algorithm = "AWS4-HMAC-SHA256";
        let credential_scope = format!("{}/{}/s3/aws4_request", date_str, self.region);
        let string_to_sign = format!(
            "{}\n{}\n{}\n{}",
            algorithm,
            date_time_str,
            credential_scope,
            hex::encode(Sha256::digest(canonical_request.as_bytes()))
        );

        let signing_key = self._signing_key(&date_str);
        let signature = hex::encode(self._hmac(&signing_key, string_to_sign.as_bytes()));

        format!(
            "{}/{}?{}&X-Amz-Signature={}",
            self.endpoint.trim_end_matches('/'),
            resource,
            query_params,
            signature
        )
    }

    pub fn generate_presigned_put_url(&self, storage_key: &str, expires_secs: u64) -> String {
        let resource = format!("{}/{}", self.bucket, storage_key);
        let expires = expires_secs.to_string();
        let now = Utc::now();
        let date_str = now.format("%Y%m%d").to_string();
        let date_time_str = now.format("%Y%m%dT%H%M%SZ").to_string();

        let host = self.host_name();
        let headers = vec![("host", host.as_str())];

        let signed_headers = "host";
        let query_params = format!(
            "X-Amz-Algorithm=AWS4-HMAC-SHA256&X-Amz-Credential={}%2F{}%2F{}%2Fs3%2Faws4_request&X-Amz-Date={}&X-Amz-Expires={}&X-Amz-SignedHeaders={}",
            &self.access_key, &date_str, &self.region, &date_time_str, &expires, signed_headers
        );

        let canonical_request = format!(
            "PUT\n/{}\n{}\n{}\n\n{}\nUNSIGNED-PAYLOAD",
            resource, query_params, self._canonical_headers(&headers), signed_headers
        );

        let algorithm = "AWS4-HMAC-SHA256";
        let credential_scope = format!("{}/{}/s3/aws4_request", date_str, self.region);
        let string_to_sign = format!(
            "{}\n{}\n{}\n{}",
            algorithm,
            date_time_str,
            credential_scope,
            hex::encode(Sha256::digest(canonical_request.as_bytes()))
        );

        let signing_key = self._signing_key(&date_str);
        let signature = hex::encode(self._hmac(&signing_key, string_to_sign.as_bytes()));

        format!(
            "{}/{}?{}&X-Amz-Signature={}",
            self.endpoint.trim_end_matches('/'),
            resource,
            query_params,
            signature
        )
    }

    pub async fn delete_object(&self, storage_key: &str) -> Result<(), String> {
        let resource = format!("{}/{}", self.bucket, storage_key);
        let url = format!("{}/{}", self.endpoint.trim_end_matches('/'), resource);
        let now = Utc::now();
        let date_str = now.format("%Y%m%d").to_string();
        let date_time_str = now.format("%Y%m%dT%H%M%SZ").to_string();

        let host = self.host_name();
        let headers = vec![("host", host.as_str())];

        let signed_headers = "host";
        let canonical_request = format!(
            "DELETE\n/{}\n\n{}\n\n{}\nUNSIGNED-PAYLOAD",
            resource, self._canonical_headers(&headers), signed_headers
        );

        let credential_scope = format!("{}/{}/s3/aws4_request", date_str, self.region);
        let string_to_sign = format!(
            "AWS4-HMAC-SHA256\n{}\n{}\n{}",
            date_time_str,
            credential_scope,
            hex::encode(Sha256::digest(canonical_request.as_bytes()))
        );

        let signing_key = self._signing_key(&date_str);
        let signature = hex::encode(self._hmac(&signing_key, string_to_sign.as_bytes()));

        let auth_header = format!(
            "AWS4-HMAC-SHA256 Credential={}/{}/s3/aws4_request, SignedHeaders={}, Signature={}",
            self.access_key, credential_scope, signed_headers, signature
        );

        let client = reqwest::Client::new();
        let response = client
            .delete(&url)
            .header("Authorization", &auth_header)
            .header("x-amz-date", &date_time_str)
            .send()
            .await
            .map_err(|e| format!("delete request failed: {}", e))?;

        if response.status().is_success() || response.status() == reqwest::StatusCode::NO_CONTENT {
            Ok(())
        } else {
            Err(format!("delete failed with status: {}", response.status()))
        }
    }

    fn host_name(&self) -> String {
        self.endpoint
            .trim_start_matches("https://")
            .trim_start_matches("http://")
            .trim_end_matches('/')
            .to_string()
    }

    fn _canonical_headers(&self, headers: &[(&str, &str)]) -> String {
        headers.iter()
            .map(|(k, v)| format!("{}:{}", k.to_lowercase(), v.trim()))
            .collect::<Vec<_>>()
            .join("\n")
    }

    fn _hmac(&self, key: &[u8], data: &[u8]) -> Vec<u8> {
        let mut mac = HmacSha256::new_from_slice(key).expect("HMAC key");
        mac.update(data);
        mac.finalize().into_bytes().to_vec()
    }

    fn _signing_key(&self, date_str: &str) -> Vec<u8> {
        let k_secret = format!("AWS4{}", self.secret_key);
        let k_date = self._hmac(k_secret.as_bytes(), date_str.as_bytes());
        let k_region = self._hmac(&k_date, self.region.as_bytes());
        let k_service = self._hmac(&k_region, b"s3");
        self._hmac(&k_service, b"aws4_request")
    }
}
