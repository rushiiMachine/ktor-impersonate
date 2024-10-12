use rquest::boring::x509::store::{X509Store, X509StoreBuilder};
use rquest::boring::x509::X509;
use std::sync::LazyLock;

/// Clone the globally cached BoringSSL [`X509Store`] for a new client.
pub fn get_cached_verify_store() -> X509Store {
	let mut store = X509StoreBuilder::new().unwrap();

	for x509_object in CERTS.objects().iter() {
		let x509_ref = x509_object.x509().unwrap();
		let x509 = x509_ref.to_owned();
		store.add_cert(x509).unwrap();
	}

	store.build()
}

static CERTS: LazyLock<X509Store> = LazyLock::new(|| {
	#[cfg(not(target_os = "android"))]
	return load_certs_desktop();

	#[cfg(target_os = "android")]
	return load_certs_mobile();
});

#[cfg(not(target_os = "android"))]
fn load_certs_desktop() -> X509Store {
	let mut verify_store = X509StoreBuilder::new().unwrap();
	let native_certs = rustls_native_certs::load_native_certs();

	if native_certs.errors.len() > 0 {
		log::debug!("Encountered errors loading root certificates: {:#?}", native_certs.errors);
	}
	log::debug!("Loaded {} platform root certificates", native_certs.certs.len());

	for cert in native_certs.certs {
		let cert = X509::from_der(&*cert).expect("Failed to re-parse certificate");
		verify_store.add_cert(cert).unwrap();
	}
	verify_store.build()
}

// TODO: use the Android TrustStore instead of hardcoding certificates
#[cfg(target_os = "android")]
fn load_certs_mobile() -> X509Store {
	let mut verify_store = X509StoreBuilder::new().unwrap();

	for cert in webpki_root_certs::TLS_SERVER_ROOT_CERTS {
		let x509 = X509::from_der(&*cert).unwrap();
		verify_store.add_cert(x509).unwrap();
	}
	verify_store.build()
}
