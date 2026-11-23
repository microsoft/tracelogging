// Copyright (c) Microsoft Corporation.
// Licensed under the MIT license.

#![allow(clippy::needless_return)]

use std::mem::size_of;

use tracelogging::_internal as tli;
use tracelogging::{Channel, Level, Opcode};

fn descriptor_fields(descriptor: &tli::EventDataDescriptor<'_>) -> (u64, u32, u32) {
    let bytes = tli::meta_as_bytes(descriptor);
    assert_eq!(bytes.len(), 16);

    return (
        u64::from_ne_bytes(bytes[0..8].try_into().unwrap()),
        u32::from_ne_bytes(bytes[8..12].try_into().unwrap()),
        u32::from_ne_bytes(bytes[12..16].try_into().unwrap()),
    );
}

fn assert_descriptor(
    descriptor: &tli::EventDataDescriptor<'_>,
    pointer: usize,
    size: u32,
    reserved: u32,
) {
    assert_eq!(
        descriptor_fields(descriptor),
        (pointer as u64, size, reserved)
    );
}

#[test]
fn event_descriptor_constructors() {
    let zero = tli::EventDescriptor::zero();
    assert_eq!(zero.id, 0);
    assert_eq!(zero.version, 0);
    assert_eq!(zero.channel, Channel::TraceClassic);
    assert_eq!(zero.level, Level::LogAlways);
    assert_eq!(zero.opcode, Opcode::Info);
    assert_eq!(zero.task, 0);
    assert_eq!(zero.keyword, 0);

    let default = tli::EventDescriptor::new(Level::Warning, 0x1234);
    assert_eq!(default.id, 0);
    assert_eq!(default.version, 0);
    assert_eq!(default.channel, Channel::TraceLogging);
    assert_eq!(default.level, Level::Warning);
    assert_eq!(default.opcode, Opcode::Info);
    assert_eq!(default.task, 0);
    assert_eq!(default.keyword, 0x1234);

    let values = tli::EventDescriptor::from_parts(
        1,
        2,
        Channel::ProviderMetadata,
        Level::Error,
        Opcode::ActivityStart,
        3,
        0x4567,
    );
    assert_eq!(values.id, 1);
    assert_eq!(values.version, 2);
    assert_eq!(values.channel, Channel::ProviderMetadata);
    assert_eq!(values.level, Level::Error);
    assert_eq!(values.opcode, Opcode::ActivityStart);
    assert_eq!(values.task, 3);
    assert_eq!(values.keyword, 0x4567);
}

#[test]
fn raw_and_value_descriptors_preserve_layout() {
    let bytes = [1, 2, 3];
    let raw = tli::EventDataDescriptor::from_raw_bytes(&bytes, 0x1234_5678);
    assert_descriptor(
        &raw,
        bytes.as_ptr() as usize,
        bytes.len() as u32,
        0x1234_5678,
    );

    let value = 0x1234_5678u32;
    let scalar = tli::EventDataDescriptor::from_value(&value);
    assert_descriptor(
        &scalar,
        &value as *const u32 as usize,
        size_of::<u32>() as u32,
        0,
    );
}

#[test]
fn cstr_descriptors_stop_before_first_nul() {
    let value = [1u16, 2, 0, 4];
    let descriptor = tli::EventDataDescriptor::from_cstr(&value);
    assert_descriptor(&descriptor, value.as_ptr() as usize, 4, 0);

    let without_nul = [1u8, 2, 3];
    let descriptor = tli::EventDataDescriptor::from_cstr(&without_nul);
    assert_descriptor(&descriptor, without_nul.as_ptr() as usize, 3, 0);

    let starts_with_nul = [0u8, 1];
    let descriptor = tli::EventDataDescriptor::from_cstr(&starts_with_nul);
    assert_descriptor(&descriptor, starts_with_nul.as_ptr() as usize, 0, 0);
}

#[test]
fn cstr_descriptors_limit_element_count() {
    let value = vec![1u8; 65_536];
    let descriptor = tli::EventDataDescriptor::from_cstr(&value);
    assert_descriptor(&descriptor, value.as_ptr() as usize, 65_535, 0);
}

#[test]
fn counted_descriptors_limit_byte_size() {
    let value = [1u16; 32_768];
    let descriptor = tli::EventDataDescriptor::from_counted(&value);
    assert_descriptor(
        &descriptor,
        value.as_ptr() as usize,
        32_767 * size_of::<u16>() as u32,
        0,
    );
    assert_eq!(tli::counted_size(&value), 32_767 * 2);

    let exact = [1u8; 65_535];
    let descriptor = tli::EventDataDescriptor::from_counted(&exact);
    assert_descriptor(&descriptor, exact.as_ptr() as usize, 65_535, 0);
    assert_eq!(tli::counted_size(&exact), 65_535);
}

#[test]
fn counted_descriptors_support_zero_sized_types() {
    let value = [(); 70_000];
    let descriptor = tli::EventDataDescriptor::from_counted(&value);
    assert_descriptor(&descriptor, value.as_ptr() as usize, 0, 0);
    assert_eq!(tli::counted_size(&value), 0);
}

#[test]
fn slice_descriptors_and_counts_limit_element_count() {
    let value = vec![1u32; 65_536];
    let descriptor = tli::EventDataDescriptor::from_slice(&value);
    assert_descriptor(
        &descriptor,
        value.as_ptr() as usize,
        65_535 * size_of::<u32>() as u32,
        0,
    );
    assert_eq!(tli::slice_count(&value), 65_535);
    assert_eq!(tli::slice_count(&value[..65_534]), 65_534);
    assert_eq!(tli::slice_count::<u32>(&[]), 0);
}
