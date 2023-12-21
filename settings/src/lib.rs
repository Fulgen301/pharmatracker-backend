use std::ops::Deref;

use base64::{engine::general_purpose, Engine};
use config::{Config, ConfigError, File};
use serde::{de::Visitor, Deserialize, Serialize};

#[derive(Clone, Debug)]
pub struct Key(String);

impl Deref for Key {
    type Target = String;

    fn deref(&self) -> &Self::Target {
        &self.0
    }
}

struct KeyVisitor;

impl<'de> Visitor<'de> for KeyVisitor {
    type Value = String;

    fn expecting(&self, formatter: &mut std::fmt::Formatter) -> std::fmt::Result {
        formatter.write_str("a RSA key in base64")
    }

    fn visit_string<E>(self, v: String) -> Result<Self::Value, E>
    where
        E: serde::de::Error,
    {
        let bytes = general_purpose::STANDARD
            .decode(v)
            .map_err(|e| serde::de::Error::custom(format!("decode: {e}")))?;

        Ok(String::from_utf8(bytes)
            .map_err(|e| serde::de::Error::custom(format!("from_utf8: {e}")))?)
    }
}

impl<'de> Deserialize<'de> for Key {
    fn deserialize<D: serde::Deserializer<'de>>(deserializer: D) -> Result<Self, D::Error> {
        Ok(Key(deserializer.deserialize_string(KeyVisitor)?))
    }
}

impl Serialize for Key {
    fn serialize<S: serde::Serializer>(&self, serializer: S) -> Result<S::Ok, S::Error> {
        serializer.serialize_str(&general_purpose::STANDARD.encode(self.0.as_bytes()))
    }
}

#[derive(Debug, Deserialize, Serialize)]
pub struct Settings {
    pub database: Database,
    pub endpoint: Endpoint,
    pub jwt: Jwt,
}

#[derive(Debug, Deserialize, Serialize)]
pub struct Database {
    pub url: String,
}

#[derive(Debug, Deserialize, Serialize)]
pub struct Endpoint {
    pub host: String,
    pub port: u16,
}

#[derive(Debug, Deserialize, Serialize)]
pub struct Jwt {
    pub access_token_private_key: Key,
    pub access_token_public_key: Key,
    /*pub refresh_token_private_key: Key,
    pub refresh_token_public_key: Key,*/
}

impl Settings {
    pub fn new(name: &str) -> Result<Settings, ConfigError> {
        //let file_public_key = File::new("examples/custom_file_format/files/public.pem", Key);
        //let file_private_key = File::new("examples/custom_file_format/files/private.pem", PemFile);

        Config::builder()
            .add_source(File::with_name(name))
            .build()?
            .try_deserialize()
    }
}
