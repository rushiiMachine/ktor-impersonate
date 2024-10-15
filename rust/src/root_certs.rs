use log::debug;
use rquest::boring;
use rquest::boring::x509::store::{X509Store, X509StoreBuilder};
use rquest::boring::x509::X509;
use std::sync::LazyLock;

type BoxError = Box<dyn std::error::Error + Send + Sync>;

/// Clone the globally cached BoringSSL [`X509Store`] for a new client.
pub fn get_cached_verify_store() -> Result<X509Store, BoxError> {
	let mut store_builder = X509StoreBuilder::new()?;

	if let Some(store) = CERTS.as_ref().ok() {
		for cert in store.objects().iter() {
			if let Some(cert) = cert.x509() {
				store_builder.add_cert(cert.to_owned())?;
			}
		}
	} else {
		// Last resort in case platform-specific load_certs() fails
		store_builder.set_default_paths()?;
	}

	Ok(store_builder.build())
}

fn build_store<I>(certs: I) -> Result<X509Store, BoxError>
where
	I: Iterator<Item=Result<X509, boring::error::ErrorStack>>,
{
	let mut valid_count = 0;
	let mut invalid_count = 0;
	let mut verify_store = X509StoreBuilder::new()?;

	for cert in certs {
		match cert {
			Ok(cert) => {
				verify_store.add_cert(cert)?;
				valid_count += 1;
			}
			Err(err) => {
				invalid_count += 1;
				debug!("Failed to parse certificate: {err:?}");
			}
		}
	}

	if valid_count == 0 && invalid_count > 0 {
		return Err("all loaded certificates are invalid".into());
	}

	Ok(verify_store.build())
}

static CERTS: LazyLock<Result<X509Store, BoxError>> = LazyLock::new(load_certs);

/// Loads certificates using [rustls_native_certs].
/// This first checks the `SSL_CERT_FILE` environment variable as an override.
/// On Windows: loads from the system certificate store
/// On macOS: loads from the keychain
/// On *nix: uses `openssl-probe` to find the certificate bundle.
#[cfg(not(any(target_os = "android", target_os = "ios")))]
fn load_certs() -> Result<X509Store, BoxError> {
	let native_certs = rustls_native_certs::load_native_certs();

	if native_certs.certs.is_empty() && !native_certs.errors.is_empty() {
		return Err("all loaded certificates are invalid".into());
	}
	if native_certs.errors.len() > 0 {
		debug!("Failed to load some native root certificates: {:#?}", native_certs.errors);
	}

	let x509_certs = native_certs
		.certs.iter()
		.map(|cert| X509::from_der(&*cert));

	build_store(x509_certs)
}

/// Load the system trusted certificates (user installed certs are ignored)
#[cfg(target_os = "android")]
fn load_certs() -> Result<X509Store, BoxError> {
	let entries = match std::fs::read_dir("/system/etc/security/cacerts") {
		Err(err) => {
			return Err(format!("failed to load system certificates: {err:?}").into())
		}
		Ok(entries) => entries,
	};
	let certs = entries
		.map(|result| result.and_then(|entry| std::fs::read(entry.path())))
		.filter_map(|result| {
			result
				.inspect_err(|err| debug!("Error opening system certificate: {:?}", err))
				.ok()
		})
		.map(|bytes| X509::from_pem(&*bytes));

	build_store(certs)
}

/// Load hardcoded webpki certificates shipped with library
// TODO: get system certificates instead
#[cfg(target_os = "ios")]
fn load_certs() -> Result<X509Store, BoxError> {
	let certs = webpki_root_certs::TLS_SERVER_ROOT_CERTS.iter()
		.map(|cert| X509::from_der(&*cert));

	build_store(certs)
}
