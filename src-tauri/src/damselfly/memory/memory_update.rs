use std::fmt::{Display, Formatter};
use std::sync::Arc;
use serde::ser::SerializeStruct;
use serde::{Deserialize, Deserializer, Serialize, Serializer};
use serde::de::{MapAccess, SeqAccess};

#[derive(PartialEq, Eq, Clone, Debug, Serialize, Deserialize)]
pub enum MemoryUpdateType {
    Allocation(Allocation),
    Free(Free)
}

impl MemoryUpdateType {
    pub fn get_absolute_address(&self) -> usize {
        match self {
            MemoryUpdateType::Allocation(allocation) => allocation.get_absolute_address(),
            MemoryUpdateType::Free(free) => free.get_absolute_address(),
        }
    }

    pub fn set_absolute_address(&mut self, new_address: usize) {
        match self {
            MemoryUpdateType::Allocation(allocation) => allocation.set_absolute_address(new_address),
            MemoryUpdateType::Free(free) => free.set_absolute_address(new_address),
        }
    }
    
    pub fn get_absolute_size(&self) -> usize {
        match self {
            MemoryUpdateType::Allocation(allocation) => allocation.get_absolute_size(),
            MemoryUpdateType::Free(free) => free.get_absolute_size(),
        }
    }
    
    pub fn set_absolute_size(&mut self, new_size: usize) {
        match self {
            MemoryUpdateType::Allocation(allocation) => allocation.set_absolute_size(new_size),
            MemoryUpdateType::Free(free) => free.set_absolute_size(new_size),
        }
    }

    pub fn get_callstack(&self) -> Arc<String> {
        match self {
            MemoryUpdateType::Allocation(allocation) => allocation.get_callstack(),
            MemoryUpdateType::Free(free) => free.get_callstack(),
        }
    }

    pub fn get_start(&self) -> usize {
        match self {
            MemoryUpdateType::Allocation(allocation) => allocation.get_absolute_address(),
            MemoryUpdateType::Free(free) => free.get_absolute_address(),
        }
    }

    pub fn get_end(&self) -> usize {
        match self {
            MemoryUpdateType::Allocation(allocation) => allocation.get_absolute_address() + allocation.get_absolute_size(),
            MemoryUpdateType::Free(free) => free.get_absolute_address() + free.get_absolute_size(),
        }
    }
    
    pub fn get_timestamp(&self) -> usize {
        match self {
            MemoryUpdateType::Allocation(allocation) => allocation.get_timestamp(),
            MemoryUpdateType::Free(free) => free.get_timestamp(),
        }
    }

    pub fn set_timestamp(&mut self, new_timestamp: usize) {
        match self {
            MemoryUpdateType::Allocation(allocation) => allocation.set_timestamp(new_timestamp),
            MemoryUpdateType::Free(free) => free.set_timestamp(new_timestamp),
        }
    }

    pub fn get_real_timestamp(&self) -> &String {
        match self {
            MemoryUpdateType::Allocation(allocation) => allocation.get_real_timestamp(),
            MemoryUpdateType::Free(free) => free.get_real_timestamp(),
        }
    }
}

impl Display for MemoryUpdateType {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        let str = match self {
            MemoryUpdateType::Allocation(allocation) =>
                allocation.to_string(),
            MemoryUpdateType::Free(free) =>
                free.to_string(),
        };
        write!(f, "{}", str)
    }
}

pub trait MemoryUpdate {
    fn get_absolute_address(&self) -> usize;
    fn set_absolute_address(&mut self, new_address: usize);
    fn get_absolute_size(&self) -> usize;
    fn set_absolute_size(&mut self, new_size: usize);
    fn get_callstack(&self) -> Arc<String>;
    fn get_timestamp(&self) -> usize;
    fn set_timestamp(&mut self, new_timestamp: usize);
    fn get_real_timestamp(&self) -> &String;
    fn wrap_in_enum(self) -> MemoryUpdateType;
}

#[derive(PartialEq, Eq, Clone, Debug)]
pub struct Allocation {
    address: usize,
    size: usize,
    callstack: Arc<String>,
    timestamp: usize,
    real_timestamp: String,
}



impl Allocation {
    pub fn new(address: usize, size: usize, callstack: Arc<String>, timestamp: usize, real_timestamp: String) -> Allocation {
        Allocation {
            address,
            size,
            callstack,
            timestamp,
            real_timestamp,
        }
    }
}

#[derive(PartialEq, Eq, Clone, Debug)]
pub struct Free {
    address: usize,
    size: usize,
    callstack: Arc<String>,
    timestamp: usize,
    real_timestamp: String,
}

impl Free {
    pub fn new(address: usize, size: usize, callstack: Arc<String>, timestamp: usize, real_timestamp: String) -> Free {
        Free {
            address,
            size,
            callstack,
            timestamp,
            real_timestamp,
        }
    }
}

impl MemoryUpdate for Allocation {
    fn get_absolute_address(&self) -> usize {
        self.address
    }

    fn set_absolute_address(&mut self, new_address: usize) {
        self.address = new_address
    }

    fn get_absolute_size(&self) -> usize {
        self.size
    }

    fn set_absolute_size(&mut self, new_size: usize) {
        self.size = new_size;
    }

    fn get_callstack(&self) -> Arc<String> {
        Arc::clone(&(self.callstack))
    }

    fn get_timestamp(&self) -> usize {
        self.timestamp
    }

    fn set_timestamp(&mut self, new_timestamp: usize) {
        self.timestamp = new_timestamp;
    }

    fn get_real_timestamp(&self) -> &String {
        &self.real_timestamp
    }

    fn wrap_in_enum(self) -> MemoryUpdateType {
        MemoryUpdateType::Allocation(self)
    }
}

impl MemoryUpdate for Free {
    fn get_absolute_address(&self) -> usize {
        self.address
    }

    fn set_absolute_address(&mut self, new_address: usize) {
        self.address = new_address
    }

    fn get_absolute_size(&self) -> usize {
        self.size
    }

    fn set_absolute_size(&mut self, new_size: usize) {
        self.size = new_size;
    }

    fn get_callstack(&self) -> Arc<String> {
        Arc::clone(&(self.callstack))
    }

    fn get_timestamp(&self) -> usize {
        self.timestamp
    }

    fn set_timestamp(&mut self, new_timestamp: usize) {
        self.timestamp = new_timestamp;
    }

    fn get_real_timestamp(&self) -> &String {
        &self.real_timestamp
    }

    fn wrap_in_enum(self) -> MemoryUpdateType {
        MemoryUpdateType::Free(self)
    }
}

impl Display for Allocation {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        let str = format!("[{} {}] ALLOC: 0x{:x} {}B",
                          self.get_timestamp(),
                          self.get_real_timestamp(),
                          self.get_absolute_address(),
                          self.get_absolute_size());
        write!(f, "{}", str)
    }
}

impl Display for Free {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        let str = format!("[{} {}] FREE: 0x{:x} {}B",
                          self.get_timestamp(),
                          self.get_real_timestamp(),
                          self.get_absolute_address(),
                          self.get_absolute_size());
        write!(f, "{}", str)
    }
}

impl Serialize for Allocation {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error> where S: Serializer {
        let mut state = serializer.serialize_struct("Allocation", 5)?;
        state.serialize_field("address", &self.address)?;
        state.serialize_field("size", &self.size)?;
        state.serialize_field("callstack", &*self.callstack)?;
        state.serialize_field("timestamp", &self.timestamp)?;
        state.serialize_field("real_timestamp", &self.real_timestamp)?;
        state.end()
    }
}

impl<'de> Deserialize<'de> for Allocation {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
        where D: Deserializer<'de> {
        enum Field { Address, Size, Callstack, Timestamp, RealTimestamp }

        impl<'de> Deserialize<'de> for Field {
            fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
                where D: Deserializer<'de> {
                struct FieldVisitor;

                impl<'de> serde::de::Visitor<'de> for FieldVisitor {
                    type Value = Field;

                    fn expecting(&self, formatter: &mut Formatter) -> std::fmt::Result {
                        formatter.write_str("Address, Size, Callstack, Timestamp, RealTimestamp")
                    }

                    fn visit_str<E>(self, value: &str) -> Result<Field, E>
                        where E: serde::de::Error, {
                        match value {
                            "address" => Ok(Field::Address),
                            "size" => Ok(Field::Size),
                            "callstack" => Ok(Field::Callstack),
                            "timestamp" => Ok(Field::Timestamp),
                            "real_timestamp" => Ok(Field::RealTimestamp),
                            _ => Err(serde::de::Error::unknown_field(value, FIELDS)),
                        }
                    }
                }
                deserializer.deserialize_identifier(FieldVisitor)
            }
        }

        struct AllocationVisitor;

        impl<'de> serde::de::Visitor<'de> for AllocationVisitor {
            type Value = Allocation;

            fn expecting(&self, formatter: &mut Formatter) -> std::fmt::Result {
                formatter.write_str("struct Allocation")
            }

            fn visit_seq<A>(self, mut seq: A) -> Result<Self::Value, A::Error> where A: SeqAccess<'de> {
                let address = seq.next_element()?
                    .ok_or_else(|| serde::de::Error::invalid_length(0, &self))?;
                let size = seq.next_element()?
                    .ok_or_else(|| serde::de::Error::invalid_length(1, &self))?;
                let callstack = seq.next_element()?
                    .ok_or_else(|| serde::de::Error::invalid_length(2, &self))?;
                let timestamp = seq.next_element()?
                    .ok_or_else(|| serde::de::Error::invalid_length(3, &self))?;
                let real_timestamp = seq.next_element()?
                    .ok_or_else(|| serde::de::Error::invalid_length(4, &self))?;
                Ok(Allocation::new(address, size, Arc::new(callstack), timestamp, real_timestamp))
            }

            fn visit_map<A>(self, mut map: A) -> Result<Self::Value, A::Error> where A: MapAccess<'de> {
                let mut address = None;
                let mut size = None;
                let mut callstack = None;
                let mut timestamp = None;
                let mut real_timestamp = None;

                while let Some(key) = map.next_key()? {
                    match key {
                        Field::Address => {
                            if address.is_some() {
                                return Err(serde::de::Error::duplicate_field("address"));
                            }
                            address = Some(map.next_value()?);
                        }
                        Field::Size => {
                            if size.is_some() {
                                return Err(serde::de::Error::duplicate_field("size"));
                            }
                            size = Some(map.next_value()?);
                        }
                        Field::Callstack => {
                            if callstack.is_some() {
                                return Err(serde::de::Error::duplicate_field("callstack"));
                            }
                            callstack = Some(map.next_value()?);
                        }
                        Field::Timestamp => {
                            if timestamp.is_some() {
                                return Err(serde::de::Error::duplicate_field("timestamp"));
                            }
                            timestamp = Some(map.next_value()?);
                        }
                        Field::RealTimestamp => {
                            if real_timestamp.is_some() {
                                return Err(serde::de::Error::duplicate_field("real_timestamp"));
                            }
                            real_timestamp = Some(map.next_value()?);
                        }
                    }
                }
                let address = address.ok_or_else(|| serde::de::Error::missing_field("address"))?;
                let size = size.ok_or_else(|| serde::de::Error::missing_field("size"))?;
                let callstack = callstack.ok_or_else(|| serde::de::Error::missing_field("callstack"))?;
                let timestamp = timestamp.ok_or_else(|| serde::de::Error::missing_field("timestamp"))?;
                let real_timestamp = real_timestamp.ok_or_else(|| serde::de::Error::missing_field("real_timestamp"))?;
                Ok(Allocation::new(address, size, Arc::new(callstack), timestamp, real_timestamp))
            }
        }

        const FIELDS: &[&str] = &["address", "size", "callstack", "timestamp", "real_timestamp"];
        deserializer.deserialize_struct("Allocation", FIELDS, AllocationVisitor)
    }
}

impl Serialize for Free {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error> where S: Serializer {
        let mut state = serializer.serialize_struct("Free", 5)?;
        state.serialize_field("address", &self.address)?;
        state.serialize_field("size", &self.size)?;
        state.serialize_field("callstack", &*self.callstack)?;
        state.serialize_field("timestamp", &self.timestamp)?;
        state.serialize_field("real_timestamp", &self.real_timestamp)?;
        state.end()
    }
}

impl<'de> Deserialize<'de> for Free {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
        where D: Deserializer<'de> {
        enum Field { Address, Size, Callstack, Timestamp, RealTimestamp }

        impl<'de> Deserialize<'de> for Field {
            fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
                where D: Deserializer<'de> {
                struct FieldVisitor;

                impl<'de> serde::de::Visitor<'de> for FieldVisitor {
                    type Value = Field;

                    fn expecting(&self, formatter: &mut Formatter) -> std::fmt::Result {
                        formatter.write_str("Address, Size, Callstack, Timestamp, RealTimestamp")
                    }

                    fn visit_str<E>(self, value: &str) -> Result<Field, E>
                        where E: serde::de::Error, {
                        match value {
                            "address" => Ok(Field::Address),
                            "size" => Ok(Field::Size),
                            "callstack" => Ok(Field::Callstack),
                            "timestamp" => Ok(Field::Timestamp),
                            "real_timestamp" => Ok(Field::RealTimestamp),
                            _ => Err(serde::de::Error::unknown_field(value, FIELDS)),
                        }
                    }
                }
                deserializer.deserialize_identifier(FieldVisitor)
            }
        }

        struct FreeVisitor;

        impl<'de> serde::de::Visitor<'de> for FreeVisitor {
            type Value = Free;

            fn expecting(&self, formatter: &mut Formatter) -> std::fmt::Result {
                formatter.write_str("struct Free")
            }

            fn visit_seq<A>(self, mut seq: A) -> Result<Self::Value, A::Error> where A: SeqAccess<'de> {
                let address = seq.next_element()?
                    .ok_or_else(|| serde::de::Error::invalid_length(0, &self))?;
                let size = seq.next_element()?
                    .ok_or_else(|| serde::de::Error::invalid_length(1, &self))?;
                let callstack = seq.next_element()?
                    .ok_or_else(|| serde::de::Error::invalid_length(2, &self))?;
                let timestamp = seq.next_element()?
                    .ok_or_else(|| serde::de::Error::invalid_length(3, &self))?;
                let real_timestamp = seq.next_element()?
                    .ok_or_else(|| serde::de::Error::invalid_length(4, &self))?;
                Ok(Free::new(address, size, Arc::new(callstack), timestamp, real_timestamp))
            }

            fn visit_map<A>(self, mut map: A) -> Result<Self::Value, A::Error> where A: MapAccess<'de> {
                let mut address = None;
                let mut size = None;
                let mut callstack = None;
                let mut timestamp = None;
                let mut real_timestamp = None;

                while let Some(key) = map.next_key()? {
                    match key {
                        Field::Address => {
                            if address.is_some() {
                                return Err(serde::de::Error::duplicate_field("address"));
                            }
                            address = Some(map.next_value()?);
                        }
                        Field::Size => {
                            if size.is_some() {
                                return Err(serde::de::Error::duplicate_field("size"));
                            }
                            size = Some(map.next_value()?);
                        }
                        Field::Callstack => {
                            if callstack.is_some() {
                                return Err(serde::de::Error::duplicate_field("callstack"));
                            }
                            callstack = Some(map.next_value()?);
                        }
                        Field::Timestamp => {
                            if timestamp.is_some() {
                                return Err(serde::de::Error::duplicate_field("timestamp"));
                            }
                            timestamp = Some(map.next_value()?);
                        }
                        Field::RealTimestamp => {
                            if real_timestamp.is_some() {
                                return Err(serde::de::Error::duplicate_field("real_timestamp"));
                            }
                            real_timestamp = Some(map.next_value()?);
                        }
                    }
                }
                let address = address.ok_or_else(|| serde::de::Error::missing_field("address"))?;
                let size = size.ok_or_else(|| serde::de::Error::missing_field("size"))?;
                let callstack = callstack.ok_or_else(|| serde::de::Error::missing_field("callstack"))?;
                let timestamp = timestamp.ok_or_else(|| serde::de::Error::missing_field("timestamp"))?;
                let real_timestamp = real_timestamp.ok_or_else(|| serde::de::Error::missing_field("real_timestamp"))?;
                Ok(Free::new(address, size, Arc::new(callstack), timestamp, real_timestamp))
            }
        }

        const FIELDS: &[&str] = &["address", "size", "callstack", "timestamp", "real_timestamp"];
        deserializer.deserialize_struct("Free", FIELDS, FreeVisitor)
    }
}