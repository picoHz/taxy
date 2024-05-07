use pkcs8::{PrivateKeyInfo, SecretDocument};
use rcgen::{
    BasicConstraints, CertificateParams, DistinguishedName, DnType, Ia5String, IsCa, KeyPair,
    SanType,
};
use sha2::{Digest, Sha256};
use std::fmt;
use std::io::{BufRead, BufReader};
use std::net::IpAddr;
use std::str::FromStr;
use taxy_api::cert::{CertInfo, CertKind, CertMetadata};
use taxy_api::error::Error;
use taxy_api::id::ShortId;
use taxy_api::subject_name::SubjectName;
use tokio_rustls::rustls::crypto::ring::sign;
use tokio_rustls::rustls::pki_types::{CertificateDer, PrivateKeyDer};
use tokio_rustls::rustls::sign::CertifiedKey;
use tracing::error;
use x509_parser::{extensions::GeneralName, time::ASN1Time};
use x509_parser::{parse_x509_certificate, prelude::X509Certificate};

pub mod acme;

#[derive(Clone)]
pub struct Cert {
    pub id: ShortId,
    pub kind: CertKind,
    pub key: Option<SecretDocument>,
    pub pem_chain: Vec<u8>,
    pub pem_key: Option<Vec<u8>>,
    pub fingerprint: String,
    pub issuer: String,
    pub root_cert: Option<String>,
    pub san: Vec<SubjectName>,
    pub not_after: ASN1Time,
    pub not_before: ASN1Time,
    pub is_ca: bool,
    pub metadata: Option<CertMetadata>,
}

impl PartialEq for Cert {
    fn eq(&self, other: &Self) -> bool {
        self.fingerprint == other.fingerprint
    }
}

impl Eq for Cert {}

impl fmt::Debug for Cert {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_struct("Cert")
            .field("id", &self.id)
            .field("fingerprint", &self.fingerprint)
            .field("issuer", &self.issuer)
            .field("root_cert", &self.root_cert)
            .field("san", &self.san)
            .field("not_after", &self.not_after)
            .field("not_before", &self.not_before)
            .field("metadata", &self.metadata)
            .finish()
    }
}

impl PartialOrd for Cert {
    fn partial_cmp(&self, other: &Self) -> Option<std::cmp::Ordering> {
        Some(
            other
                .not_before
                .partial_cmp(&self.not_before)
                .unwrap()
                .then_with(|| self.not_after.partial_cmp(&other.not_after).unwrap())
                .then_with(|| self.fingerprint.cmp(&other.fingerprint)),
        )
    }
}

impl Cert {
    pub fn id(&self) -> ShortId {
        self.id
    }

    pub fn info(&self) -> CertInfo {
        CertInfo {
            id: self.id,
            kind: self.kind,
            fingerprint: self.fingerprint.clone(),
            issuer: self.issuer.clone(),
            root_cert: self.root_cert.clone(),
            san: self.san.clone(),
            not_after: self.not_after.timestamp(),
            not_before: self.not_before.timestamp(),
            is_ca: self.is_ca,
            has_private_key: self.key.is_some(),
            metadata: self.metadata.clone(),
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

    pub fn new(
        kind: CertKind,
        pem_chain: Vec<u8>,
        pem_key: Option<Vec<u8>>,
    ) -> Result<Self, Error> {
        let key = if let Some(pem_key) = &pem_key {
            let key_pem =
                std::str::from_utf8(pem_key).map_err(|_| Error::FailedToReadPrivateKey)?;
            let (_, key) =
                SecretDocument::from_pem(key_pem).map_err(|_| Error::FailedToReadPrivateKey)?;
            Some(key)
        } else {
            None
        };
        let chain_meta = pem_chain.as_slice();
        let mut meta_read = BufReader::new(chain_meta);
        let mut comment = String::new();
        meta_read
            .read_line(&mut comment)
            .map_err(|_| Error::FailedToReadCertificate)?;

        let metadata: Option<CertMetadata> = serde_qs::from_str(
            comment
                .trim_start_matches(|c: char| c == '#' || c.is_whitespace())
                .trim_end(),
        )
        .ok();

        let mut chain = pem_chain.as_slice();
        let certs = rustls_pemfile::certs(&mut chain)
            .map(|cert| cert.map_err(|_| Error::FailedToReadCertificate))
            .collect::<Result<Vec<_>, _>>()?;

        let der = certs
            .first()
            .ok_or(Error::FailedToReadCertificate)?
            .as_ref();
        let mut hasher = Sha256::new();
        hasher.update(der);
        let id = hasher.finalize();
        let mut short_id = [0; 7];
        short_id.copy_from_slice(&id[..7]);
        let fingerprint = hex::encode(id);

        let parsed_chain = parse_chain(&certs)?;
        let x509 = parsed_chain.first().ok_or(Error::FailedToReadCertificate)?;
        let san = x509
            .subject_alternative_name()
            .into_iter()
            .flatten()
            .flat_map(|name| &name.value.general_names)
            .filter_map(|name| match name {
                GeneralName::DNSName(name) => SubjectName::from_str(name).ok(),
                GeneralName::IPAddress(ip) => match ip.len() {
                    4 => {
                        let addr = [ip[0], ip[1], ip[2], ip[3]];
                        Some(SubjectName::IPAddress(IpAddr::V4(addr.into())))
                    }
                    16 => {
                        let mut addr = [0; 16];
                        addr.copy_from_slice(ip);
                        Some(SubjectName::IPAddress(IpAddr::V6(addr.into())))
                    }
                    _ => None,
                },
                _ => None,
            })
            .collect();

        let not_after = x509.validity().not_after;
        let not_before = x509.validity().not_before;
        let is_ca = x509.is_ca();

        let issuer = x509.issuer().to_string();
        let root_cert = parsed_chain
            .last()
            .filter(|_| chain.len() > 1)
            .map(|cert| cert.subject().to_string());

        Ok(Self {
            id: short_id.into(),
            kind,
            fingerprint,
            key,
            pem_chain,
            pem_key,
            issuer,
            root_cert,
            san,
            not_after,
            not_before,
            is_ca,
            metadata,
        })
    }

    pub fn new_ca() -> Result<Self, Error> {
        let mut distinguished_name = DistinguishedName::new();
        distinguished_name.push(DnType::CommonName, "Taxy CA");
        let mut params = CertificateParams::default();
        params.is_ca = IsCa::Ca(BasicConstraints::Unconstrained);
        params.distinguished_name = distinguished_name;

        let keypair =
            KeyPair::generate().map_err(|_| Error::FailedToGerateSelfSignedCertificate)?;
        let cert = match params.self_signed(&keypair) {
            Ok(cert) => cert,
            Err(err) => {
                error!(%err);
                return Err(Error::FailedToGerateSelfSignedCertificate);
            }
        };

        let pem_chain = cert.pem().into_bytes();
        let pem_key = keypair.serialize_pem().into_bytes();
        Self::new(CertKind::Root, pem_chain, Some(pem_key))
    }

    pub fn new_self_signed(san: &[SubjectName], ca: &Cert) -> Result<Self, Error> {
        let ca_pem =
            std::str::from_utf8(&ca.pem_chain).map_err(|_| Error::FailedToReadPrivateKey)?;
        let pem_key = ca.pem_key.as_ref().ok_or(Error::FailedToReadPrivateKey)?;
        let key_pem = std::str::from_utf8(pem_key).map_err(|_| Error::FailedToReadPrivateKey)?;
        let ca_keypair = KeyPair::from_pem(key_pem).map_err(|_| Error::FailedToReadPrivateKey)?;
        let ca_params = CertificateParams::from_ca_cert_pem(ca_pem)
            .map_err(|_| Error::FailedToGerateSelfSignedCertificate)?;

        let ca_cert = match ca_params.self_signed(&ca_keypair) {
            Ok(cert) => cert,
            Err(err) => {
                error!(%err);
                return Err(Error::FailedToGerateSelfSignedCertificate);
            }
        };

        let mut params = CertificateParams::default();
        for name in san {
            let name = if let SubjectName::IPAddress(ip) = name {
                SanType::IpAddress(*ip)
            } else {
                let name = Ia5String::from_str(&name.to_string())
                    .map_err(|_| Error::FailedToGerateSelfSignedCertificate)?;
                SanType::DnsName(name)
            };
            params.subject_alt_names.push(name);
        }

        let common_name = san
            .iter()
            .map(|name| name.to_string())
            .next()
            .unwrap_or_else(|| "Taxy Cert".into());
        let mut distinguished_name = DistinguishedName::new();
        distinguished_name.push(DnType::CommonName, common_name);
        params.distinguished_name = distinguished_name;

        let keypair =
            KeyPair::generate().map_err(|_| Error::FailedToGerateSelfSignedCertificate)?;
        let cert = match params.signed_by(&keypair, &ca_cert, &ca_keypair) {
            Ok(cert) => cert,
            Err(err) => {
                error!(%err);
                return Err(Error::FailedToGerateSelfSignedCertificate);
            }
        };

        let pem_chain = format!("{}\r\n{}", cert.pem(), ca_cert.pem()).into_bytes();
        let pem_key = keypair.serialize_pem().into_bytes();
        Self::new(CertKind::Server, pem_chain, Some(pem_key))
    }

    pub fn certified_key(&self) -> Result<CertifiedKey, Error> {
        match self.certified_impl() {
            Ok(certified) => Ok(certified),
            Err(err) => {
                error!(%err);
                Err(Error::FailedToReadPrivateKey)
            }
        }
    }

    pub fn certificates(&self) -> Result<Vec<CertificateDer<'static>>, Error> {
        let mut chain = self.pem_chain.as_slice();
        rustls_pemfile::certs(&mut chain)
            .map(|cert| {
                cert.map(|cert| cert.to_owned())
                    .map_err(|_| Error::FailedToReadCertificate)
            })
            .collect()
    }

    fn certified_impl(&self) -> anyhow::Result<CertifiedKey> {
        let key = self.key.as_ref().ok_or(Error::FailedToReadPrivateKey)?;
        let key = key
            .decode_msg::<PrivateKeyInfo>()
            .map_err(|err| anyhow::anyhow!("{err}"))?;
        let key =
            PrivateKeyDer::try_from(key.private_key).map_err(|err| anyhow::anyhow!("{err}"))?;
        let signing_key = sign::any_supported_type(&key).map_err(|err| anyhow::anyhow!("{err}"))?;
        let chain = self.certificates()?;
        Ok(CertifiedKey::new(chain, signing_key))
    }
}

fn parse_chain<'a>(chain: &'a [CertificateDer]) -> Result<Vec<X509Certificate<'a>>, Error> {
    chain
        .iter()
        .map(|data| {
            parse_x509_certificate(data.as_ref())
                .map(|(_, cert)| cert)
                .map_err(|_| Error::FailedToReadCertificate)
        })
        .collect()
}

#[cfg(test)]
mod test {

    #[test]
    fn test_self_signed() {
        use super::*;

        let san = [SubjectName::from_str("localhost").unwrap()];
        let ca = Cert::new_ca().unwrap();
        let cert = Cert::new_self_signed(&san, &ca).unwrap();
        assert_eq!(cert.san, san);
    }
}
