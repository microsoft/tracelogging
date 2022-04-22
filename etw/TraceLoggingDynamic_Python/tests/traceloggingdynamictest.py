import traceloggingdynamic as tld
import unittest as ut
import uuid
import timeit

class TraceLoggingDynamicTest(ut.TestCase):

    def test_speed(self):

        p = tld.Provider(b'TraceLoggingDynamicTest', throw_on_error=True)
        print(str(p) + ' enabled: ' + str(p.is_enabled()))
        e = tld.EventBuilder()

        def BenchmarkEvent():
            e.reset(b'Benchmark')
            e.add_str8_bytes(b'f1', b'val')
            e.add_int32(b'f2', 10)
            p.write(e)

        number = 15000
        time = timeit.timeit(BenchmarkEvent, number = number)
        print('Benchmark event:   N =', f'{number};', 'events/s =', f'{number/time:,.0f}.')

        time = timeit.timeit(p.is_enabled, number = number)
        print('Benchmark is_enabled: N =', f'{number};', 'calls/s  =', f'{number/time:,.0f}.')

    def test_providerid_from_name(self):

        self.assertEqual(
            tld.providerid_from_name('MyProvider'),
            uuid.UUID('b3864c38-4273-58c5-545b-8b3608343471'))
        self.assertEqual(
            tld.providerid_from_name('MYPROVIDER'),
            uuid.UUID('b3864c38-4273-58c5-545b-8b3608343471'))
        self.assertEqual(
            tld.providerid_from_name_bytes_utf8(b'MYPROVIDER'),
            uuid.UUID('b3864c38-4273-58c5-545b-8b3608343471'))

    def test_Provider(self):

        with self.assertRaises(ValueError):
            p = tld.Provider(b'')
        with self.assertRaises(ValueError):
            p = tld.Provider(b'Hello\x00')

        p = tld.Provider(b'TestProvider')
        self.assertTrue(p.is_open())
        p.close()
        self.assertFalse(p.is_open())
        self.assertEqual(p.id, uuid.UUID('97c801ee-c28b-5bb6-2ae4-11e18fe6137a'))

        p = tld.Provider(b'TestProvider', id = uuid.NAMESPACE_DNS)
        self.assertEqual(p.id, uuid.NAMESPACE_DNS)
        self.assertFalse(p.is_enabled())

        self.assertEqual(
            str(p).upper(),
            'Provider "TestProvider" {6ba7b810-9dad-11d1-80b4-00c04fd430c8}'.upper())

        # Make sure the destructor works.
        p = []
        with self.assertRaises(OSError): # Limit of ~2000 providers per process.
            for x in range(5000): p.append(tld.Provider(b'TestProvider', throw_on_error = True))
        self.assertGreater(len(p), 100)
        with self.assertRaises(OSError): # Still at limit.
            tld.Provider(b'TestProvider', throw_on_error = True)
        p2 = tld.Provider(b'TestProvider') # Still at limit, but it shouldn't throw.
        self.assertFalse(p2.is_open()) # Handle is null.
        del p # Close a bunch of handles.
        p2 = tld.Provider(b'TestProvider', throw_on_error = True) # Now we should be ok again.
        self.assertTrue(p2.is_open())

    def test_EventBuilder(self):

        p = tld.Provider(b'TraceLoggingDynamicTest', throw_on_error=True)
        e = tld.EventBuilder()
        with self.assertRaises(ValueError): e.check_state() # Starts in error state.
        with self.assertRaises(AssertionError): e.reset(b'Hello\x00')

        e.reset(b'default')
        p.write(e)

        e.reset(b'4v2o1t123c0l3k11', id=4, version=2, opcode=1, task=123, channel=0, level=3, keyword=0x11)
        p.write(e)

        e.reset(b'tag0xFE00000', tag=0xFE00000)
        p.write(e)

        e.reset(b'tag0xFEDC000', tag=0xFEDC000)
        p.write(e)

        e.reset(b'tag0xFEDCBAF', tag=0xFEDCBAF)
        p.write(e)

        e.reset(b'fieldtag')
        e.add_uint8(b'0xFE00000', 0, tag = 0xFE00000)
        e.add_uint8(b'0xFEDC000', 0, tag = 0xFEDC000)
        e.add_uint8(b'0xFEDCBAF', 0, tag = 0xFEDCBAF)
        p.write(e)

        e.reset(b'outtypes')
        e.add_uint8(b'default100', 100)
        e.add_uint8(b'string65', 65, out_type=tld.OutType.STRING)
        e.add_uint8(b'bool1', 1, out_type=tld.OutType.BOOLEAN)
        e.add_uint8(b'hex100', 100, out_type=tld.OutType.HEX)
        p.write(e)

        e.reset(b'structs')
        e.add_uint8(b'start', 0)
        e.add_struct(b'struct1', 2, tag = 0xFE00000)
        e.  add_uint8(b'nested1', 1)
        e.  add_struct(b'struct2', 1)
        e.      add_uint8(b'nested2', 2)
        p.write(e)

        e.reset(b'zstrs-L4-kFF', 4, 0xff)
        e.add_uint8(b'A', 65, out_type=tld.OutType.STRING)
        e.add_zstr16(b'zstr16-', '')
        e.add_zstr16(b'zstr16-a', 'a')
        e.add_zstr16(b'zstr16-0', '\0')
        e.add_zstr16(b'zstr16-a0', 'a\0')
        e.add_zstr16(b'zstr16-0a', '\0a')
        e.add_zstr16(b'zstr16-a0a', 'a\0a')
        e.add_zstr16_bytes_utf16le(b'zstr16b-', b'')
        e.add_zstr16_bytes_utf16le(b'zstr16b-a', b'a\0')
        e.add_zstr16_bytes_utf16le(b'zstr16b-0', b'\0\0')
        e.add_zstr16_bytes_utf16le(b'zstr16b-a0', b'a\0\0\0')
        e.add_zstr16_bytes_utf16le(b'zstr16b-0a', b'\0\0a\0')
        e.add_zstr16_bytes_utf16le(b'zstr16b-a0a', b'a\0\0\0a\0')
        e.add_zstr8(b'zstr8-', '')
        e.add_zstr8(b'zstr8-a', 'a')
        e.add_zstr8(b'zstr8-0', '\0')
        e.add_zstr8(b'zstr8-a0', 'a\0')
        e.add_zstr8(b'zstr8-0a', '\0a')
        e.add_zstr8(b'zstr8-a0a', 'a\0a')
        e.add_zstr8_bytes(b'zstr8b-', b'')
        e.add_zstr8_bytes(b'zstr8b-a', b'a')
        e.add_zstr8_bytes(b'zstr8b-0', b'\0')
        e.add_zstr8_bytes(b'zstr8b-a0', b'a\0')
        e.add_zstr8_bytes(b'zstr8b-0a', b'\0a')
        e.add_zstr8_bytes(b'zstr8b-a0a', b'a\0a')
        e.add_uint8(b'A', 65, out_type=tld.OutType.STRING)
        p.write(e)

        e.reset(b'scalars')
        e.add_uint8(b'A', 65, out_type=tld.OutType.STRING)
        e.add_zstr16(b'zstr16', 'zutf16')
        e.add_zstr16_bytes_utf16le(b'zstr16b', 'zutf16b'.encode('utf_16_le'))
        e.add_zstr8(b'zstr8', 'zutf8')
        e.add_zstr8_bytes(b'zstr8b', b'zcp1252')
        e.add_int8(b'int8', -8)
        e.add_uint8(b'uint8', 8)
        e.add_int16(b'int16', -16)
        e.add_uint16(b'uint16', 16)
        e.add_int32(b'int32', -32)
        e.add_uint32(b'uint32', 32)
        e.add_int64(b'int64', -64)
        e.add_uint64(b'uint64', 64)
        e.add_intptr(b'intptr', -3264)
        e.add_uintptr(b'uintptr', 3264)
        e.add_float32(b'float32', 3.2)
        e.add_float64(b'float64', 6.4)
        e.add_bool32(b'bool32-f', False)
        e.add_bool32(b'bool32-t', True)
        e.add_binary_bytes(b'binaryb', b'0123')
        e.add_guid(b'guid', tld.providerid_from_name('sample'))
        e.add_guid_bytes_le(b'guidb', tld.providerid_from_name('sample').bytes_le)
        e.add_filetime_int64(b'filetimeu', 0x01d7ace794497cb5)
        e.add_systemtime_bytes(b'systemtimeb', bytes([0xd0, 0x07, 1, 0, 1, 0, 2, 0, 3, 0, 4, 0, 5, 0, 6, 0]))
        e.add_sid_bytes(b'sidb', bytes([1, 1, 0, 0, 0, 0, 0, 0, 1, 0, 0, 0]))
        e.add_hexint32(b'hexint32', 0xdeadbeef)
        e.add_hexint64(b'hexint64', 0xdeadbeeffeeef000)
        e.add_hexintptr(b'hexintptr', 0x1234)
        e.add_str16(b'str16', 'utf16')
        e.add_str16_bytes_utf16le(b'str16b', 'utf16b'.encode('utf_16_le'))
        e.add_str8(b'str8', 'utf8')
        e.add_str8_bytes(b'str8b', b'cp1252')
        e.add_cbinary_bytes(b'cbinaryb', b'0123')
        e.add_uint8(b'A', 65, out_type=tld.OutType.STRING)
        p.write(e)

        self.do_sequence(p, e, b'zstr16', e.add_zstr16_seq, 'zutf16')
        self.do_sequence(p, e, b'zstr16b', e.add_zstr16_bytes_utf16le_seq, 'zutf16b'.encode('utf_16_le'))
        self.do_sequence(p, e, b'zstr8', e.add_zstr8_seq, 'zutf8')
        self.do_sequence(p, e, b'zstr8b', e.add_zstr8_bytes_seq, b'zcp1252')
        self.do_sequence(p, e, b'int8', e.add_int8_seq, -8)
        self.do_sequence(p, e, b'uint8', e.add_uint8_seq, 8)
        self.do_sequence(p, e, b'int16', e.add_int16_seq, -16)
        self.do_sequence(p, e, b'uint16', e.add_uint16_seq, 16)
        self.do_sequence(p, e, b'int32', e.add_int32_seq, -32)
        self.do_sequence(p, e, b'uint32', e.add_uint32_seq, 32)
        self.do_sequence(p, e, b'int64', e.add_int64_seq, -64)
        self.do_sequence(p, e, b'uint64', e.add_uint64_seq, 64)
        self.do_sequence(p, e, b'intptr', e.add_intptr_seq, -3264)
        self.do_sequence(p, e, b'uintptr', e.add_uintptr_seq, 3264)
        self.do_sequence(p, e, b'float32', e.add_float32_seq, 3.2)
        self.do_sequence(p, e, b'float64', e.add_float64_seq, 6.4)
        self.do_sequence(p, e, b'bool32-f', e.add_bool32_seq, False)
        self.do_sequence(p, e, b'bool32-t', e.add_bool32_seq, True)
        self.do_sequence(p, e, b'guid', e.add_guid_seq, tld.providerid_from_name('sample'))
        self.do_sequence(p, e, b'guidb', e.add_guid_bytes_le_seq, tld.providerid_from_name('sample').bytes_le)
        self.do_sequence(p, e, b'filetimeu', e.add_filetime_int64_seq, 0x01d7ace794497cb5)
        self.do_sequence(p, e, b'systemtimeb', e.add_systemtime_bytes_seq, bytes([0xd0, 0x07, 1, 0, 1, 0, 2, 0, 3, 0, 4, 0, 5, 0, 6, 0]))
        self.do_sequence(p, e, b'sidb', e.add_sid_bytes_seq, bytes([1, 1, 0, 0, 0, 0, 0, 0, 1, 0, 0, 0]))
        self.do_sequence(p, e, b'hexint32', e.add_hexint32_seq, 0xdeadbeef)
        self.do_sequence(p, e, b'hexint64', e.add_hexint64_seq, 0xdeadbeeffeeef000)
        self.do_sequence(p, e, b'hexintptr', e.add_hexintptr_seq, 0x1234)
        self.do_sequence(p, e, b'str16', e.add_str16_seq, 'utf16')
        self.do_sequence(p, e, b'str16b', e.add_str16_bytes_utf16le_seq, 'utf16b'.encode('utf_16_le'))
        self.do_sequence(p, e, b'str8', e.add_str8_seq, 'utf8')
        self.do_sequence(p, e, b'str8b', e.add_str8_bytes_seq, b'cp1252')
        self.do_sequence(p, e, b'cbinaryb', e.add_cbinary_bytes_seq, b'0123')

    def do_sequence(self, p, e, name_bytes, method, val):
        e.reset(name_bytes)
        e.add_uint8(b'A', 65, out_type=tld.OutType.STRING)
        method(b'0', ())
        method(b'1', (val,))
        method(b'2', (val,val))
        e.add_uint8(b'A', 65, out_type=tld.OutType.STRING)
        p.write(e)

ut.main()
