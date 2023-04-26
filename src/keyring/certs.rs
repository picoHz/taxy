use std::str::FromStr;

use crate::error::Error;
use rcgen::{CertificateParams, DistinguishedName, DnType, SanType};
use rustls_pemfile::Item;
use serde_derive::{Deserialize, Serialize};
use sha2::{Digest, Sha256};
use tokio_rustls::rustls::{Certificate, PrivateKey};
use tracing::error;
use x509_parser::{extensions::GeneralName, time::ASN1Time};
use x509_parser::{parse_x509_certificate, prelude::X509Certificate};

use super::SubjectName;

const CERT_ID_LENGTH: usize = 20;

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Cert {
    pub id: String,
    pub chain: Vec<Certificate>,
    pub key: PrivateKey,
    pub raw_chain: Vec<u8>,
    pub raw_key: Vec<u8>,
    pub fingerprint: String,
    pub issuer: String,
    pub root_cert: Option<String>,
    pub san: Vec<SubjectName>,
    pub not_after: ASN1Time,
    pub not_before: ASN1Time,
    pub is_self_signed: bool,
}

impl Cert {
    pub fn id(&self) -> &str {
        &self.id
    }

    pub fn info(&self) -> CertInfo {
        CertInfo {
            id: self.id.clone(),
            fingerprint: self.fingerprint.clone(),
            issuer: self.issuer.clone(),
            root_cert: self.root_cert.clone(),
            san: self.san.clone(),
            not_after: self.not_after.timestamp(),
            not_before: self.not_before.timestamp(),
            is_self_signed: self.is_self_signed,
        }
    }

    pub fn is_valid(&self) -> bool {
        let now = ASN1Time::now();
        self.not_before <= now && now <= self.not_after
    }

    pub fn has_subject_name(&self, name: &SubjectName) -> bool {
        for san in &self.san {
            if match (san, name) {
                (SubjectName::DnsName(c), SubjectName::DnsName(n)) => c == n,
                (SubjectName::WildcardDnsName(c), SubjectName::DnsName(n)) => {
                    c == n.trim_start_matches(|c| c != '.').trim_start_matches('.')
                }
                (SubjectName::WildcardDnsName(c), SubjectName::WildcardDnsName(n)) => c == n,
                (SubjectName::IPAddress(c), SubjectName::IPAddress(n)) => c == n,
                _ => false,
            } {
                return true;
            }
        }
        false
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
pub struct CertInfo {
    pub id: String,
    pub fingerprint: String,
    pub issuer: String,
    pub root_cert: Option<String>,
    pub san: Vec<SubjectName>,
    pub not_after: i64,
    pub not_before: i64,
    pub is_self_signed: bool,
}

#[derive(Debug, Clone, PartialEq, Eq, Deserialize)]
pub struct SelfSignedCertRequest {
    pub san: Vec<SubjectName>,
}

impl Cert {
    pub fn new(raw_chain: Vec<u8>, raw_key: Vec<u8>) -> Result<Self, Error> {
        let mut chain = raw_chain.as_slice();
        let mut key = raw_key.as_slice();
        let chain =
            rustls_pemfile::certs(&mut chain).map_err(|_| Error::FailedToReadCertificate)?;
        let chain = chain.into_iter().map(Certificate).collect::<Vec<_>>();

        let mut privkey = None;
        while let Some(key) =
            rustls_pemfile::read_one(&mut key).map_err(|_| Error::FailedToReadPrivateKey)?
        {
            match key {
                Item::RSAKey(key) | Item::PKCS8Key(key) | Item::ECKey(key) => {
                    if privkey.is_none() {
                        privkey = Some(PrivateKey(key));
                    }
                }
                _ => {}
            }
        }

        let key = privkey.ok_or(Error::FailedToReadPrivateKey)?;
        let der = &chain.first().ok_or(Error::FailedToReadCertificate)?.0;
        let mut hasher = Sha256::new();
        hasher.update(der);
        let fingerprint = hex::encode(hasher.finalize());

        let parsed_chain = parse_chain(&chain)?;
        let is_self_signed = parsed_chain
            .iter()
            .any(|cert| cert.issuer() == cert.subject());

        let x509 = parsed_chain.first().ok_or(Error::FailedToReadCertificate)?;
        let san = x509
            .subject_alternative_name()
            .into_iter()
            .flatten()
            .flat_map(|name| &name.value.general_names)
            .filter_map(|name| match name {
                GeneralName::DNSName(name) => SubjectName::from_str(name).ok(),
                _ => None,
            })
            .collect();

        let not_after = x509.validity().not_after;
        let not_before = x509.validity().not_before;

        let issuer = x509.issuer().to_string();
        let root_cert = parsed_chain
            .last()
            .filter(|_| chain.len() > 1)
            .map(|cert| cert.subject().to_string());

        Ok(Cert {
            id: fingerprint[..CERT_ID_LENGTH].to_string(),
            fingerprint,
            chain,
            key,
            raw_chain,
            raw_key,
            issuer,
            root_cert,
            san,
            not_after,
            not_before,
            is_self_signed,
        })
    }

    pub fn new_self_signed(req: &SelfSignedCertRequest) -> Result<Self, Error> {
        let mut distinguished_name = DistinguishedName::new();
        distinguished_name.push(DnType::CommonName, "taxy self signed cert");

        let mut params = CertificateParams::default();
        params.subject_alt_names = req
            .san
            .iter()
            .map(|name| {
                if let SubjectName::IPAddress(ip) = name {
                    SanType::IpAddress(*ip)
                } else {
                    SanType::DnsName(name.to_string())
                }
            })
            .collect();
        params.distinguished_name = distinguished_name;

        let cert = match rcgen::Certificate::from_params(params) {
            Ok(cert) => cert,
            Err(err) => {
                error!(?err);
                return Err(Error::FailedToGerateSelfSignedCertificate);
            }
        };

        let raw_chain = cert
            .serialize_pem()
            .map_err(|_| Error::FailedToGerateSelfSignedCertificate)?
            .into_bytes();
        let raw_key = cert.serialize_private_key_pem().into_bytes();

        Self::new(raw_chain, raw_key)
    }
}

fn parse_chain(chain: &[Certificate]) -> Result<Vec<X509Certificate>, Error> {
    let mut certs = Vec::new();
    for data in chain {
        let (_, cert) =
            parse_x509_certificate(&data.0).map_err(|_| Error::FailedToReadCertificate)?;
        certs.push(cert);
    }
    Ok(certs)
}

#[cfg(test)]
mod test {
    #[test]
    fn test_self_signed() {
        use super::*;

        let req = SelfSignedCertRequest {
            san: vec![SubjectName::from_str("localhost").unwrap()],
        };
        let cert = Cert::new_self_signed(&req).unwrap();
        assert_eq!(cert.san, req.san);
    }
}
