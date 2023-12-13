use serde::Deserialize;

use crate::adapter::{AdapterBaseMsg, AdapterRequestMsg, BaseExecuteMsg};

impl<'de, Request> Deserialize<'de> for AdapterRequestMsg<Request>
where
    Request: Deserialize<'de>,
{
    #[allow(clippy::too_many_lines)]
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: cosmwasm_schema::serde::Deserializer<'de>,
    {
        use std::fmt;
        use std::marker::PhantomData;

        use serde::de::{self, Deserializer, MapAccess, Visitor};
        use serde_cw_value::Value;

        #[derive(Debug)]
        enum Field {
            ProxyAddress,
            Request(Value),
        }

        const FIELDS: &[&str] = &["proxy_address"];

        impl<'de> Deserialize<'de> for Field {
            fn deserialize<D>(deserializer: D) -> Result<Field, D::Error>
            where
                D: Deserializer<'de>,
            {
                struct FieldVisitor;

                impl<'de> Visitor<'de> for FieldVisitor {
                    type Value = Field;

                    fn expecting(&self, formatter: &mut fmt::Formatter) -> fmt::Result {
                        formatter.write_str("`proxy_address`")
                    }

                    fn visit_str<E>(self, v: &str) -> Result<Self::Value, E>
                    where
                        E: de::Error,
                    {
                        match v {
                            "proxy_address" => Ok(Field::ProxyAddress),
                            _ => Ok(Field::Request(Value::String(v.to_owned()))),
                        }
                    }

                    fn visit_borrowed_str<E>(self, v: &'de str) -> Result<Self::Value, E>
                    where
                        E: de::Error,
                    {
                        match v {
                            "proxy_address" => Ok(Field::ProxyAddress),
                            _ => Ok(Field::Request(Value::String(v.to_owned()))),
                        }
                    }

                    fn visit_bytes<E>(self, v: &[u8]) -> Result<Self::Value, E>
                    where
                        E: de::Error,
                    {
                        match v {
                            b"proxy_address" => Ok(Field::ProxyAddress),
                            _ => Ok(Field::Request(Value::Bytes(v.to_owned()))),
                        }
                    }

                    fn visit_byte_buf<E>(self, v: Vec<u8>) -> Result<Self::Value, E>
                    where
                        E: de::Error,
                    {
                        match v.as_slice() {
                            b"proxy_address" => Ok(Field::ProxyAddress),
                            _ => Ok(Field::Request(Value::Bytes(v))),
                        }
                    }

                    fn visit_borrowed_bytes<E>(self, v: &'de [u8]) -> Result<Self::Value, E>
                    where
                        E: de::Error,
                    {
                        match v {
                            b"proxy_address" => Ok(Field::ProxyAddress),
                            _ => Ok(Field::Request(Value::Bytes(v.to_owned()))),
                        }
                    }
                }

                deserializer.deserialize_identifier(FieldVisitor)
            }
        }

        struct AdapterRequestVisitor<Request> {
            _m: PhantomData<Request>,
        }

        impl<'de, Request> Visitor<'de> for AdapterRequestVisitor<Request>
        where
            Request: Deserialize<'de>,
        {
            type Value = AdapterRequestMsg<Request>;

            fn expecting(&self, formatter: &mut fmt::Formatter) -> fmt::Result {
                formatter.write_str("struct AdapterRequestMsg")
            }

            fn visit_map<V>(self, mut map: V) -> Result<Self::Value, V::Error>
            where
                V: MapAccess<'de>,
            {
                let mut proxy_address = None;
                let mut request = vec![];

                while let Some(key) = map.next_key()? {
                    match key {
                        Field::ProxyAddress => {
                            if proxy_address.is_some() {
                                return Err(de::Error::duplicate_field("proxy_address"));
                            }
                            proxy_address = map.next_value()?;
                        }
                        Field::Request(key) => {
                            let value = map.next_value()?;
                            request.push((key, value));
                        }
                    }
                }

                Ok(AdapterRequestMsg {
                    proxy_address,
                    request: Request::deserialize(Value::Map(request.into_iter().collect()))
                        .map_err(|err| de::Error::custom(err.to_string()))?,
                })
            }
        }

        deserializer.deserialize_struct(
            "AdapterRequestMsg",
            FIELDS,
            AdapterRequestVisitor { _m: PhantomData },
        )
    }
}

// BaseExecuteMsg

impl<'de> Deserialize<'de> for BaseExecuteMsg {
    #[allow(clippy::too_many_lines)]
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: cosmwasm_schema::serde::Deserializer<'de>,
    {
        use std::fmt;

        use serde::de::{self, Deserializer, MapAccess, Visitor};
        use serde_cw_value::Value;

        #[derive(Debug)]
        enum Field {
            ProxyAddress,
            Msg(Value),
        }

        const FIELDS: &[&str] = &["proxy_address"];

        impl<'de> Deserialize<'de> for Field {
            fn deserialize<D>(deserializer: D) -> Result<Field, D::Error>
            where
                D: Deserializer<'de>,
            {
                struct FieldVisitor;

                impl<'de> Visitor<'de> for FieldVisitor {
                    type Value = Field;

                    fn expecting(&self, formatter: &mut fmt::Formatter) -> fmt::Result {
                        formatter.write_str("`proxy_address`")
                    }

                    fn visit_str<E>(self, v: &str) -> Result<Self::Value, E>
                    where
                        E: de::Error,
                    {
                        match v {
                            "proxy_address" => Ok(Field::ProxyAddress),
                            _ => Ok(Field::Msg(Value::String(v.to_owned()))),
                        }
                    }

                    fn visit_borrowed_str<E>(self, v: &'de str) -> Result<Self::Value, E>
                    where
                        E: de::Error,
                    {
                        match v {
                            "proxy_address" => Ok(Field::ProxyAddress),
                            _ => Ok(Field::Msg(Value::String(v.to_owned()))),
                        }
                    }

                    fn visit_bytes<E>(self, v: &[u8]) -> Result<Self::Value, E>
                    where
                        E: de::Error,
                    {
                        match v {
                            b"proxy_address" => Ok(Field::ProxyAddress),
                            _ => Ok(Field::Msg(Value::Bytes(v.to_owned()))),
                        }
                    }

                    fn visit_byte_buf<E>(self, v: Vec<u8>) -> Result<Self::Value, E>
                    where
                        E: de::Error,
                    {
                        match v.as_slice() {
                            b"proxy_address" => Ok(Field::ProxyAddress),
                            _ => Ok(Field::Msg(Value::Bytes(v))),
                        }
                    }

                    fn visit_borrowed_bytes<E>(self, v: &'de [u8]) -> Result<Self::Value, E>
                    where
                        E: de::Error,
                    {
                        match v {
                            b"proxy_address" => Ok(Field::ProxyAddress),
                            _ => Ok(Field::Msg(Value::Bytes(v.to_owned()))),
                        }
                    }
                }

                deserializer.deserialize_identifier(FieldVisitor)
            }
        }

        struct BaseExecuteVisitor {}

        impl<'de> Visitor<'de> for BaseExecuteVisitor {
            type Value = BaseExecuteMsg;

            fn expecting(&self, formatter: &mut fmt::Formatter) -> fmt::Result {
                formatter.write_str("struct BaseExecuteMsg")
            }

            fn visit_map<V>(self, mut map: V) -> Result<Self::Value, V::Error>
            where
                V: MapAccess<'de>,
            {
                let mut proxy_address = None;
                let mut msg = vec![];

                while let Some(key) = map.next_key()? {
                    match key {
                        Field::ProxyAddress => {
                            if proxy_address.is_some() {
                                return Err(de::Error::duplicate_field("proxy_address"));
                            }
                            proxy_address = map.next_value()?;
                        }
                        Field::Msg(key) => {
                            let value = map.next_value()?;
                            msg.push((key, value));
                        }
                    }
                }

                Ok(BaseExecuteMsg {
                    proxy_address,
                    msg: AdapterBaseMsg::deserialize(Value::Map(msg.into_iter().collect()))
                        .map_err(|err| de::Error::custom(err.to_string()))?,
                })
            }
        }

        deserializer.deserialize_struct("BaseExecuteMsg", FIELDS, BaseExecuteVisitor {})
    }
}
