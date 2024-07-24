// This file is generated by rust-protobuf 2.23.0. Do not edit
// @generated

// https://github.com/rust-lang/rust-clippy/issues/702
#![allow(unknown_lints)]
#![allow(clippy::all)]

#![allow(unused_attributes)]
#![cfg_attr(rustfmt, rustfmt::skip)]

#![allow(dead_code)]
#![allow(missing_docs)]
#![allow(non_camel_case_types)]
#![allow(non_snake_case)]
#![allow(non_upper_case_globals)]
#![allow(trivial_casts)]
#![allow(unused_imports)]
#![allow(unused_results)]
//! Generated file from `src/response.proto`

/// Generated files are compatible only with the same version
/// of protobuf runtime.
// const _PROTOBUF_VERSION_CHECK: () = ::protobuf::VERSION_2_23_0;

#[derive(PartialEq,Clone,Default)]
pub struct MsgInstantiateContractResponse {
    // message fields
    pub contract_address: ::std::string::String,
    pub data: ::std::vec::Vec<u8>,
    // special fields
    pub unknown_fields: ::protobuf::UnknownFields,
    pub cached_size: ::protobuf::CachedSize,
}

impl<'a> ::std::default::Default for &'a MsgInstantiateContractResponse {
    fn default() -> &'a MsgInstantiateContractResponse {
        <MsgInstantiateContractResponse as ::protobuf::Message>::default_instance()
    }
}

impl MsgInstantiateContractResponse {
    pub fn new() -> MsgInstantiateContractResponse {
        ::std::default::Default::default()
    }

    // string contract_address = 1;


    pub fn get_contract_address(&self) -> &str {
        &self.contract_address
    }
    pub fn clear_contract_address(&mut self) {
        self.contract_address.clear();
    }

    // Param is passed by value, moved
    pub fn set_contract_address(&mut self, v: ::std::string::String) {
        self.contract_address = v;
    }

    // Mutable pointer to the field.
    // If field is not initialized, it is initialized with default value first.
    pub fn mut_contract_address(&mut self) -> &mut ::std::string::String {
        &mut self.contract_address
    }

    // Take field
    pub fn take_contract_address(&mut self) -> ::std::string::String {
        ::std::mem::replace(&mut self.contract_address, ::std::string::String::new())
    }

    // bytes data = 2;


    pub fn get_data(&self) -> &[u8] {
        &self.data
    }
    pub fn clear_data(&mut self) {
        self.data.clear();
    }

    // Param is passed by value, moved
    pub fn set_data(&mut self, v: ::std::vec::Vec<u8>) {
        self.data = v;
    }

    // Mutable pointer to the field.
    // If field is not initialized, it is initialized with default value first.
    pub fn mut_data(&mut self) -> &mut ::std::vec::Vec<u8> {
        &mut self.data
    }

    // Take field
    pub fn take_data(&mut self) -> ::std::vec::Vec<u8> {
        ::std::mem::replace(&mut self.data, ::std::vec::Vec::new())
    }
}

impl ::protobuf::Message for MsgInstantiateContractResponse {
    fn is_initialized(&self) -> bool {
        true
    }

    fn merge_from(&mut self, is: &mut ::protobuf::CodedInputStream<'_>) -> ::protobuf::ProtobufResult<()> {
        while !is.eof()? {
            let (field_number, wire_type) = is.read_tag_unpack()?;
            match field_number {
                1 => {
                    ::protobuf::rt::read_singular_proto3_string_into(wire_type, is, &mut self.contract_address)?;
                },
                2 => {
                    ::protobuf::rt::read_singular_proto3_bytes_into(wire_type, is, &mut self.data)?;
                },
                _ => {
                    ::protobuf::rt::read_unknown_or_skip_group(field_number, wire_type, is, self.mut_unknown_fields())?;
                },
            };
        }
        ::std::result::Result::Ok(())
    }

    // Compute sizes of nested messages
    #[allow(unused_variables)]
    fn compute_size(&self) -> u32 {
        let mut my_size = 0;
        if !self.contract_address.is_empty() {
            my_size += ::protobuf::rt::string_size(1, &self.contract_address);
        }
        if !self.data.is_empty() {
            my_size += ::protobuf::rt::bytes_size(2, &self.data);
        }
        my_size += ::protobuf::rt::unknown_fields_size(self.get_unknown_fields());
        self.cached_size.set(my_size);
        my_size
    }

    fn write_to_with_cached_sizes(&self, os: &mut ::protobuf::CodedOutputStream<'_>) -> ::protobuf::ProtobufResult<()> {
        if !self.contract_address.is_empty() {
            os.write_string(1, &self.contract_address)?;
        }
        if !self.data.is_empty() {
            os.write_bytes(2, &self.data)?;
        }
        os.write_unknown_fields(self.get_unknown_fields())?;
        ::std::result::Result::Ok(())
    }

    fn get_cached_size(&self) -> u32 {
        self.cached_size.get()
    }

    fn get_unknown_fields(&self) -> &::protobuf::UnknownFields {
        &self.unknown_fields
    }

    fn mut_unknown_fields(&mut self) -> &mut ::protobuf::UnknownFields {
        &mut self.unknown_fields
    }

    fn as_any(&self) -> &dyn (::std::any::Any) {
        self as &dyn (::std::any::Any)
    }
    fn as_any_mut(&mut self) -> &mut dyn (::std::any::Any) {
        self as &mut dyn (::std::any::Any)
    }
    fn into_any(self: ::std::boxed::Box<Self>) -> ::std::boxed::Box<dyn (::std::any::Any)> {
        self
    }

    fn descriptor(&self) -> &'static ::protobuf::reflect::MessageDescriptor {
        Self::descriptor_static()
    }

    fn new() -> MsgInstantiateContractResponse {
        MsgInstantiateContractResponse::new()
    }

    fn descriptor_static() -> &'static ::protobuf::reflect::MessageDescriptor {
        static descriptor: ::protobuf::rt::LazyV2<::protobuf::reflect::MessageDescriptor> = ::protobuf::rt::LazyV2::INIT;
        descriptor.get(|| {
            let mut fields = ::std::vec::Vec::new();
            fields.push(::protobuf::reflect::accessor::make_simple_field_accessor::<_, ::protobuf::types::ProtobufTypeString>(
                "contract_address",
                |m: &MsgInstantiateContractResponse| { &m.contract_address },
                |m: &mut MsgInstantiateContractResponse| { &mut m.contract_address },
            ));
            fields.push(::protobuf::reflect::accessor::make_simple_field_accessor::<_, ::protobuf::types::ProtobufTypeBytes>(
                "data",
                |m: &MsgInstantiateContractResponse| { &m.data },
                |m: &mut MsgInstantiateContractResponse| { &mut m.data },
            ));
            ::protobuf::reflect::MessageDescriptor::new_pb_name::<MsgInstantiateContractResponse>(
                "MsgInstantiateContractResponse",
                fields,
                file_descriptor_proto()
            )
        })
    }

    fn default_instance() -> &'static MsgInstantiateContractResponse {
        static instance: ::protobuf::rt::LazyV2<MsgInstantiateContractResponse> = ::protobuf::rt::LazyV2::INIT;
        instance.get(MsgInstantiateContractResponse::new)
    }
}

impl ::protobuf::Clear for MsgInstantiateContractResponse {
    fn clear(&mut self) {
        self.contract_address.clear();
        self.data.clear();
        self.unknown_fields.clear();
    }
}

impl ::std::fmt::Debug for MsgInstantiateContractResponse {
    fn fmt(&self, f: &mut ::std::fmt::Formatter<'_>) -> ::std::fmt::Result {
        ::protobuf::text_format::fmt(self, f)
    }
}

impl ::protobuf::reflect::ProtobufValue for MsgInstantiateContractResponse {
    fn as_ref(&self) -> ::protobuf::reflect::ReflectValueRef {
        ::protobuf::reflect::ReflectValueRef::Message(self)
    }
}

static file_descriptor_proto_data: &'static [u8] = b"\
    \n\x12src/response.proto\"_\n\x1eMsgInstantiateContractResponse\x12)\n\
    \x10contract_address\x18\x01\x20\x01(\tR\x0fcontractAddress\x12\x12\n\
    \x04data\x18\x02\x20\x01(\x0cR\x04dataJ\xf8\x02\n\x06\x12\x04\0\0\x08\
    \x03\n\x08\n\x01\x0c\x12\x03\0\0\x12\n_\n\x02\x04\0\x12\x04\x03\0\x08\
    \x03\x1aS\x20MsgInstantiateContractResponse\x20defines\x20the\x20Msg/Ins\
    tantiateContract\x20response\x20type.\n\n\n\n\x03\x04\0\x01\x12\x03\x03\
    \x08&\nR\n\x04\x04\0\x02\0\x12\x03\x05\x04\x20\x1aE\x20ContractAddress\
    \x20is\x20the\x20bech32\x20address\x20of\x20the\x20new\x20contract\x20in\
    stance.\n\n\x0c\n\x05\x04\0\x02\0\x05\x12\x03\x05\x04\n\n\x0c\n\x05\x04\
    \0\x02\0\x01\x12\x03\x05\x0b\x1b\n\x0c\n\x05\x04\0\x02\0\x03\x12\x03\x05\
    \x1e\x1f\nO\n\x04\x04\0\x02\x01\x12\x03\x07\x04\x13\x1aB\x20Data\x20cont\
    ains\x20base64-encoded\x20bytes\x20to\x20returned\x20from\x20the\x20cont\
    ract\n\n\x0c\n\x05\x04\0\x02\x01\x05\x12\x03\x07\x04\t\n\x0c\n\x05\x04\0\
    \x02\x01\x01\x12\x03\x07\n\x0e\n\x0c\n\x05\x04\0\x02\x01\x03\x12\x03\x07\
    \x11\x12b\x06proto3\
";

static file_descriptor_proto_lazy: ::protobuf::rt::LazyV2<::protobuf::descriptor::FileDescriptorProto> = ::protobuf::rt::LazyV2::INIT;

fn parse_descriptor_proto() -> ::protobuf::descriptor::FileDescriptorProto {
    ::protobuf::Message::parse_from_bytes(file_descriptor_proto_data).unwrap()
}

pub fn file_descriptor_proto() -> &'static ::protobuf::descriptor::FileDescriptorProto {
    file_descriptor_proto_lazy.get(|| {
        parse_descriptor_proto()
    })
}
