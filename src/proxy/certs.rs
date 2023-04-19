use super::tls::SubjectName;
use std::{
    fs,
    io::BufReader,
    path::{Path, PathBuf},
    time::Duration,
};
use tokio_rustls::rustls::{Certificate, PrivateKey};
use tracing::{debug, error};
use x509_parser::{
    parse_x509_certificate,
    pem::parse_x509_pem,
    prelude::{ParsedExtension, X509Certificate},
};

const MINIMUM_EXPIRY: Duration = Duration::from_secs(60 * 60 * 24);
const CERT_FILE_PATTERN: &str = "*.{pem,crt,cer}";
const KEY_FILE_PATTERN: &str = "*.key";
const MAX_SEARCH_DEPTH: usize = 4;

pub fn load_single_file(base: &Path) -> anyhow::Result<(Vec<Certificate>, PrivateKey)> {
    use rustls_pemfile::Item;

    let walker =
        globwalk::GlobWalkerBuilder::from_patterns(base, &[CERT_FILE_PATTERN, KEY_FILE_PATTERN])
            .max_depth(MAX_SEARCH_DEPTH)
            .build()?
            .into_iter()
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
