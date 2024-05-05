use std::fmt::Display;

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Usd<'a>(rusty_money::Money<'a, rusty_money::iso::Currency>);

impl Usd<'_> {
    pub fn from_str(s: &str) -> Result<Self, rusty_money::MoneyError> {
        Ok(Self(rusty_money::Money::from_str(
            s,
            rusty_money::iso::USD,
        )?))
    }
}

impl Display for Usd<'_> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.0)
    }
}

impl serde::Serialize for Usd<'_> {
    fn serialize<S: serde::Serializer>(&self, serializer: S) -> Result<S::Ok, S::Error> {
        self.to_string().serialize(serializer)
    }
}

impl<'de> serde::Deserialize<'de> for Usd<'_> {
    fn deserialize<D: serde::Deserializer<'de>>(deserializer: D) -> Result<Self, D::Error> {
        let s = String::deserialize(deserializer)?;
        // remove quotes
        let s = s.trim_matches('"');
        // remove dollar sign
        let s = &s[1..];
        Self::from_str(s).map_err(serde::de::Error::custom)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_usd_from_str() {
        let usd = Usd::from_str("5.00").unwrap();
        assert_eq!(usd.to_string(), "$5.00");
    }

    #[test]
    fn test_usd_serialize() {
        let usd = Usd::from_str("5.00").unwrap();
        let serialized = serde_json::to_string(&usd).unwrap();
        assert_eq!(serialized, "\"$5.00\"");
    }

    #[test]
    fn test_usd_deserialize() {
        let deserialized: Usd = serde_json::from_str("\"$5.00\"").unwrap();
        assert_eq!(deserialized.to_string(), "$5.00");
    }

    #[test]
    fn test_serde() {
        let usd = Usd::from_str("5.00").unwrap();
        let serialized = serde_json::to_string(&usd).unwrap();
        let deserialized: Usd = serde_json::from_str(&serialized).unwrap();
        assert_eq!(usd, deserialized);
    }
}
