use rcgen::{CertificateParams, DistinguishedName, DnType, SanType};
use rustls_pemfile::Item;
use serde_derive::{Deserialize, Serialize};
use sha2::{Digest, Sha256};
use std::{
    fs,
    io::BufReader,
    path::{Path, PathBuf},
    str::FromStr,
    time::Duration,
};
use tokio_rustls::rustls::{Certificate, PrivateKey};
use tracing::{debug, error};
use x509_parser::{extensions::GeneralName, prelude::X509Error};
use x509_parser::{
    parse_x509_certificate,
    pem::parse_x509_pem,
    prelude::{ParsedExtension, X509Certificate},
};

mod subject_name;
pub use subject_name::*;

use crate::error::Error;

const MINIMUM_EXPIRY: Duration = Duration::from_secs(60 * 60 * 24);
const CERT_FILE_PATTERN: &str = "*.{pem,crt,cer}";
const KEY_FILE_PATTERN: &str = "*.key";
const MAX_SEARCH_DEPTH: usize = 4;
const CERT_ID_LENGTH: usize = 20;

pub fn load_single_file(base: &Path) -> anyhow::Result<(Vec<Certificate>, PrivateKey)> {
    let walker =
        globwalk::GlobWalkerBuilder::from_patterns(base, &[CERT_FILE_PATTERN, KEY_FILE_PATTERN])
            .max_depth(MAX_SEARCH_DEPTH)
            .build()?
            .filter_map(Result::ok);

    let mut certs = Vec::new();
    let mut privkey = None;
    for pem in walker {
        let keyfile = std::fs::File::open(pem.path())?;
        let mut reader = BufReader::new(keyfile);

        while let Some(key) = rustls_pemfile::read_one(&mut reader)? {
            match key {
                Item::X509Certificate(cert) => certs.push(Certificate(cert)),
                Item::RSAKey(key) | Item::PKCS8Key(key) | Item::ECKey(key) => {
                    if privkey.is_none() {
                        privkey = Some(PrivateKey(key));
                    }
                }
                _ => {}
            }
        }
    }

    let privkey = match privkey {
        Some(key) => key,
        None => anyhow::bail!("no key found in {:?}", base),
    };

    Ok((certs, privkey))
}

pub fn search_cert_from_name(base: &Path, names: &[SubjectName]) -> Option<PathBuf> {
    let walker = globwalk::GlobWalkerBuilder::from_patterns(base, &[CERT_FILE_PATTERN])
        .max_depth(MAX_SEARCH_DEPTH)
        .build();

    let walker = match walker {
        Ok(walker) => walker,
        Err(e) => {
            error!(path = ?base, "{:?}", e);
            return None;
        }
    };

    for pem in walker.into_iter().filter_map(Result::ok) {
        match scan_certificate_san(pem.path(), names) {
            Ok(true) => return Some(pem.path().to_owned()),
            Err(e) => {
                debug!(path = ?pem.path(), "{:?}", e);
            }
            _ => {}
        }
    }

    None
}

fn scan_certificate_san(path: &Path, names: &[SubjectName]) -> anyhow::Result<bool> {
    let data = fs::read(path)?;
    let (_, pem) = parse_x509_pem(&data)?;
    let (_, cert) = parse_x509_certificate(&pem.contents)?;
    match cert.validity().time_to_expiration() {
        Some(expiry) if expiry >= MINIMUM_EXPIRY => Ok(has_subject_name(&cert, names)),
        _ => Ok(false),
    }
}

fn has_subject_name(cert: &X509Certificate, names: &[SubjectName]) -> bool {
    for ex in cert.extensions() {
        if let ParsedExtension::SubjectAlternativeName(san) = ex.parsed_extension() {
            return names
                .iter()
                .all(|name| san.general_names.iter().any(|g| name.test(g)));
        }
    }
    false
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Cert {
    pub info: CertInfo,
    pub chain: Vec<Certificate>,
    pub key: PrivateKey,
    pub raw_chain: Vec<u8>,
    pub raw_key: Vec<u8>,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
pub struct CertInfo {
    pub id: String,
    pub fingerprint: String,
    pub san: Vec<SubjectName>,
}

#[derive(Debug, Clone, PartialEq, Eq, Deserialize)]
pub struct SelfSignedCertRequest {
    pub san: Vec<SubjectName>,
}

impl CertInfo {
    fn new(der: &[u8]) -> Result<Self, X509Error> {
        let mut hasher = Sha256::new();
        hasher.update(der);
        let fingerprint = hex::encode(hasher.finalize());
        let (_, x509) = parse_x509_certificate(der)?;
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
        Ok(CertInfo {
            id: fingerprint[..CERT_ID_LENGTH].to_string(),
            fingerprint,
            san,
        })
    }
}

impl Cert {
    pub fn new(raw_chain: Vec<u8>, raw_key: Vec<u8>) -> Result<Self, Error> {
        let mut chain = raw_chain.as_slice();
        let mut key = raw_key.as_slice();
        let chain =
            rustls_pemfile::certs(&mut chain).map_err(|_| Error::FailedToReadCertificate)?;
        let info = CertInfo::new(chain.last().ok_or(Error::FailedToReadCertificate)?)
            .map_err(|_| Error::FailedToReadCertificate)?;
        let chain = chain.into_iter().map(Certificate).collect();

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
        Ok(Cert {
            info,
            chain,
            key,
            raw_chain,
            raw_key,
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

        let der = cert
            .serialize_der()
            .map_err(|_| Error::FailedToGerateSelfSignedCertificate)?;

        let info = CertInfo::new(&der).map_err(|_| Error::FailedToReadCertificate)?;

        let chain = vec![Certificate(der)];
        let key = PrivateKey(cert.serialize_private_key_der());

        let raw_chain = cert
            .serialize_pem()
            .map_err(|_| Error::FailedToGerateSelfSignedCertificate)?
            .into_bytes();
        let raw_key = cert.serialize_private_key_pem().into_bytes();

        Ok(Self {
            info,
            chain,
            key,
            raw_chain,
            raw_key,
        })
    }
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
        assert_eq!(cert.info.san, req.san);
    }
}
