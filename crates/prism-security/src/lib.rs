use std::collections::HashMap;
use std::fmt;
use std::sync::{Arc, Mutex};

#[derive(Clone, Debug, Eq, PartialEq, Hash)]
pub struct SecretRef(String);

impl SecretRef {
    pub fn new(value: impl Into<String>) -> Result<Self, SecretError> {
        let value = value.into();
        if value.is_empty()
            || value.len() > 128
            || !value.chars().all(|character| {
                character.is_ascii_alphanumeric() || matches!(character, '.' | '_' | ':' | '-')
            })
        {
            return Err(SecretError::InvalidReference);
        }
        Ok(Self(value))
    }

    pub fn as_str(&self) -> &str {
        &self.0
    }
}

impl fmt::Display for SecretRef {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(formatter, "SecretRef({})", self.0)
    }
}

#[derive(Clone, PartialEq, Eq)]
pub struct SecretValue(String);

impl SecretValue {
    pub fn new(value: impl Into<String>) -> Result<Self, SecretError> {
        let value = value.into();
        if value.is_empty() {
            return Err(SecretError::EmptySecret);
        }
        Ok(Self(value))
    }

    pub fn expose(&self) -> &str {
        &self.0
    }
}

impl fmt::Debug for SecretValue {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        formatter.write_str("SecretValue([REDACTED])")
    }
}

#[derive(Debug, thiserror::Error)]
pub enum SecretError {
    #[error("invalid secret reference")]
    InvalidReference,
    #[error("secret value is empty")]
    EmptySecret,
    #[error("secret not found")]
    NotFound,
    #[error("OS credential store error")]
    OsStore,
}

pub trait SecretStore: Send + Sync {
    fn set(&self, reference: &SecretRef, value: &SecretValue) -> Result<(), SecretError>;
    fn get(&self, reference: &SecretRef) -> Result<SecretValue, SecretError>;
    fn delete(&self, reference: &SecretRef) -> Result<(), SecretError>;
}

#[derive(Clone, Default)]
pub struct MockSecretStore {
    values: Arc<Mutex<HashMap<SecretRef, SecretValue>>>,
}

impl MockSecretStore {
    pub fn new() -> Self {
        Self::default()
    }
}

impl SecretStore for MockSecretStore {
    fn set(&self, reference: &SecretRef, value: &SecretValue) -> Result<(), SecretError> {
        self.values
            .lock()
            .map_err(|_| SecretError::OsStore)?
            .insert(reference.clone(), value.clone());
        Ok(())
    }

    fn get(&self, reference: &SecretRef) -> Result<SecretValue, SecretError> {
        self.values
            .lock()
            .map_err(|_| SecretError::OsStore)?
            .get(reference)
            .cloned()
            .ok_or(SecretError::NotFound)
    }

    fn delete(&self, reference: &SecretRef) -> Result<(), SecretError> {
        let removed = self
            .values
            .lock()
            .map_err(|_| SecretError::OsStore)?
            .remove(reference)
            .is_some();
        if removed {
            Ok(())
        } else {
            Err(SecretError::NotFound)
        }
    }
}

pub struct KeyringSecretStore {
    service: String,
}

impl KeyringSecretStore {
    pub fn new(service: impl Into<String>) -> Result<Self, SecretError> {
        let service = service.into();
        if service.is_empty() || service.len() > 128 {
            return Err(SecretError::InvalidReference);
        }
        Ok(Self { service })
    }

    fn entry(&self, reference: &SecretRef) -> Result<keyring::Entry, SecretError> {
        keyring::Entry::new(&self.service, reference.as_str()).map_err(|_| SecretError::OsStore)
    }
}

impl SecretStore for KeyringSecretStore {
    fn set(&self, reference: &SecretRef, value: &SecretValue) -> Result<(), SecretError> {
        self.entry(reference)?
            .set_password(value.expose())
            .map_err(|_| SecretError::OsStore)
    }

    fn get(&self, reference: &SecretRef) -> Result<SecretValue, SecretError> {
        match self.entry(reference)?.get_password() {
            Ok(value) => SecretValue::new(value),
            Err(keyring::Error::NoEntry) => Err(SecretError::NotFound),
            Err(_) => Err(SecretError::OsStore),
        }
    }

    fn delete(&self, reference: &SecretRef) -> Result<(), SecretError> {
        match self.entry(reference)?.delete_credential() {
            Ok(()) => Ok(()),
            Err(keyring::Error::NoEntry) => Err(SecretError::NotFound),
            Err(_) => Err(SecretError::OsStore),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn secret_reference_is_validated() {
        assert!(SecretRef::new("account:123-personal").is_ok());
        assert!(SecretRef::new("").is_err());
        assert!(SecretRef::new("account/secret").is_err());
        assert!(SecretRef::new("x".repeat(129)).is_err());
    }

    #[test]
    fn secret_value_is_redacted_in_debug_output() {
        let secret = SecretValue::new("top-secret").unwrap();
        assert_eq!(format!("{secret:?}"), "SecretValue([REDACTED])");
        assert!(!format!("{secret:?}").contains("top-secret"));
        assert!(SecretValue::new("").is_err());
    }

    #[test]
    fn os_secret_store_backend_can_be_constructed() {
        assert!(KeyringSecretStore::new("QuotaPrism.test").is_ok());
        assert!(KeyringSecretStore::new("").is_err());
    }

    #[test]
    fn mock_store_adds_updates_reads_and_deletes_secrets() {
        let store = MockSecretStore::new();
        let reference = SecretRef::new("account:demo").unwrap();
        store
            .set(&reference, &SecretValue::new("first").unwrap())
            .unwrap();
        assert_eq!(store.get(&reference).unwrap().expose(), "first");

        store
            .set(&reference, &SecretValue::new("second").unwrap())
            .unwrap();
        assert_eq!(store.get(&reference).unwrap().expose(), "second");

        store.delete(&reference).unwrap();
        assert!(matches!(store.get(&reference), Err(SecretError::NotFound)));
        assert!(matches!(
            store.delete(&reference),
            Err(SecretError::NotFound)
        ));
    }

    #[test]
    fn mock_store_does_not_expose_missing_secrets() {
        let store = MockSecretStore::new();
        let reference = SecretRef::new("account:missing").unwrap();
        assert!(matches!(store.get(&reference), Err(SecretError::NotFound)));
    }
}
