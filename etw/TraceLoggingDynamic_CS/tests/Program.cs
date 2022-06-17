using Microsoft.TraceLoggingDynamic;
using System.Security.Principal;
using System;
using System.Diagnostics;
using System.Text;

namespace TraceLoggingDynamicTest
{
    class Program
    {
        static void Main()
        {
            var utf8 = Encoding.UTF8;

            if (EventProvider.GetGuidForName("MyProvider") != new Guid("b3864c38-4273-58c5-545b-8b3608343471") ||
                EventProvider.GetGuidForName("MYPROVIDER") != new Guid("b3864c38-4273-58c5-545b-8b3608343471") ||
                EventProvider.GetGuidForName("myprovider") != new Guid("b3864c38-4273-58c5-545b-8b3608343471"))
            {
                Console.WriteLine("ERROR: Bad GetGuidForName.");
            }

            var eb = new EventBuilder();
            using (var p = new EventProvider("TraceLoggingDynamicTest"))
            {
                Console.WriteLine("{0} enabled: {1}", p, p.IsEnabled());

                eb.Reset("Default");
                p.Write(eb);

                eb.Reset("4v2o6t123c0l3k11",
                    new EventDescriptor { Id = 4, Version = 2, Opcode = EventOpcode.Reply, Task = 123, Channel = 0, Level = EventLevel.Warning, Keyword = 0x11 });
                p.Write(eb);

                eb.Reset("tag0xFE00000", EventLevel.Verbose, 1, 0xFE00000);
                p.Write(eb);

                eb.Reset("tag0xFEDC000", EventLevel.Verbose, 1, 0xFEDC000);
                p.Write(eb);

                eb.Reset("tag0xFEDCBAF", EventLevel.Verbose, 1, 0xFEDCBAF);
                p.Write(eb);

                eb.Reset("fieldtag");
                eb.AddUInt8("0xFE00000", 0, EventOutType.Default, 0xFE00000);
                eb.AddUInt8("0xFEDC000", 0, EventOutType.Default, 0xFEDC000);
                eb.AddUInt8("0xFEDCBAF", 0, EventOutType.Default, 0xFEDCBAF);
                p.Write(eb);

                eb.Reset("outtypes");
                eb.AddUInt8("default100", 100);
                eb.AddUInt8("string65", 65, EventOutType.String);
                eb.AddUInt8("bool1", 1, EventOutType.Boolean);
                eb.AddUInt8("hex100", 100, EventOutType.Hex);
                p.Write(eb);

                eb.Reset("structs");
                eb.AddUInt8("start", 0);
                eb.AddStruct("struct1", 2, 0xFE00000);
                eb.AddUInt8("nested1", 1);
                eb.AddStruct("struct2", 1);
                eb.AddUInt8("nested2", 2);
                p.Write(eb);

                eb.Reset("zstrs-L4-kFF", EventLevel.Info, 0xff);
                eb.AddUInt8("A", 65, EventOutType.String);
                eb.AddUnicodeString("zstr16-", "");
                eb.AddUnicodeString("zstr16-a", "a");
                eb.AddUnicodeString("zstr16-0", "\0");
                eb.AddUnicodeString("zstr16-a0", "a\0");
                eb.AddUnicodeString("zstr16-0a", "\0a");
                eb.AddUnicodeString("zstr16-a0a", "a\0a");
                eb.AddAnsiString("zstr8-", utf8.GetBytes(""));
                eb.AddAnsiString("zstr8-a", utf8.GetBytes("a"));
                eb.AddAnsiString("zstr8-0", utf8.GetBytes("\0"));
                eb.AddAnsiString("zstr8-a0", utf8.GetBytes("a\0"));
                eb.AddAnsiString("zstr8-0a", utf8.GetBytes("\0a"));
                eb.AddAnsiString("zstr8-a0a", utf8.GetBytes("a\0a"));
                eb.AddUInt8("A", 65, EventOutType.String);
                p.Write(eb);

                ValidateBeginCount(p, eb, "UnicodeString", eb.AddUnicodeString, eb.AddUnicodeString, eb.AddUnicodeStringArray, "zutf16");
                ValidateBeginCount(p, eb, "AnsiString", eb.AddAnsiString, eb.AddAnsiString, eb.AddAnsiStringArray, utf8.GetBytes("zutf8"));
                Validate<sbyte>(p, eb, "Int8", eb.AddInt8, eb.AddInt8Array, -8);
                Validate<byte>(p, eb, "UInt8", eb.AddUInt8, eb.AddUInt8Array, 8);
                Validate<short>(p, eb, "Int16", eb.AddInt16, eb.AddInt16Array, -16);
                Validate<ushort>(p, eb, "UInt16", eb.AddUInt16, eb.AddUInt16Array, 16);
                Validate<int>(p, eb, "Int32", eb.AddInt32, eb.AddInt32Array, -32);
                Validate<uint>(p, eb, "UInt32", eb.AddUInt32, eb.AddUInt32Array, 32u);
                Validate<long>(p, eb, "Int64", eb.AddInt64, eb.AddInt64Array, -64);
                Validate<ulong>(p, eb, "UInt64", eb.AddUInt64, eb.AddUInt64Array, 64);
                Validate(p, eb, "IntPtr", eb.AddIntPtr, eb.AddIntPtrArray, (IntPtr)(-3264));
                Validate(p, eb, "UIntPtr", eb.AddUIntPtr, eb.AddUIntPtrArray, (UIntPtr)3264);
                Validate(p, eb, "Float32", eb.AddFloat32, eb.AddFloat32Array, 3.2f);
                Validate(p, eb, "Float64", eb.AddFloat64, eb.AddFloat64Array, 6.4);
                Validate(p, eb, "Bool32", eb.AddBool32, eb.AddBool32Array, 0);
                Validate(p, eb, "Bool32", eb.AddBool32, eb.AddBool32Array, 1);
                ValidateBeginCount(p, eb, "Binary", eb.AddBinary, eb.AddBinary, eb.AddCountedBinaryArray, utf8.GetBytes("0123"));
                Validate(p, eb, "Guid", eb.AddGuid, eb.AddGuidArray, EventProvider.GetGuidForName("sample"));
                Validate(p, eb, "FileTime", eb.AddFileTime, eb.AddFileTimeArray, 0x01d7ace794497cb5);
                Validate(p, eb, "FileTime", eb.AddFileTime, eb.AddFileTimeArray, new DateTime(2022, 12, 25, 1, 2, 3, DateTimeKind.Utc));
                Validate(p, eb, "SystemTime", eb.AddSystemTime, eb.AddSystemTimeArray, new short[] { 2022, 1, 1, 2, 3, 4, 5, 6 });
                Validate(p, eb, "Sid", eb.AddSid, eb.AddSidArray, new byte[] { 1, 1, 0, 0, 0, 0, 0, 0, 1, 0, 0, 0 });
                Validate(p, eb, "HexInt32", eb.AddHexInt32, eb.AddHexInt32Array, unchecked((int)0xdeadbeef));
                Validate(p, eb, "HexInt32", eb.AddHexInt32, eb.AddHexInt32Array, 0xdeadbeef);
                Validate(p, eb, "HexInt64", eb.AddHexInt64, eb.AddHexInt64Array, unchecked((long)0xdeadbeeffeeef000));
                Validate(p, eb, "HexInt64", eb.AddHexInt64, eb.AddHexInt64Array, 0xdeadbeeffeeef000);
                Validate(p, eb, "HexIntPtr", eb.AddHexIntPtr, eb.AddHexIntPtrArray, (IntPtr)0x1234);
                Validate(p, eb, "HexIntPtr", eb.AddHexIntPtr, eb.AddHexIntPtrArray, (UIntPtr)0x1234);
                ValidateBeginCount(p, eb, "CountedString", eb.AddCountedString, eb.AddCountedString, eb.AddCountedStringArray, "utf16");
                ValidateBeginCount(p, eb, "CountedAnsiString", eb.AddCountedAnsiString, eb.AddCountedAnsiString, eb.AddCountedAnsiStringArray, utf8.GetBytes("utf8"));
                ValidateBeginCount(p, eb, "CountedBinary", eb.AddCountedBinary, eb.AddCountedBinary, eb.AddCountedBinaryArray, utf8.GetBytes("0123"));

                var val = utf8.GetBytes("val");
                var N = p.IsEnabled() ? 10000 : 1000000;
                var timer = Stopwatch.StartNew();
                for (int i = 0; i < N; i++)
                {
                    eb.Reset("Benchmark");
                    eb.AddCountedAnsiString("f1", val);
                    eb.AddInt32("f2", i);
                    p.Write(eb);
                }

                timer.Stop();

                Console.WriteLine("Benchmark event: N = {0}; events/s = {1}",
                    N, N / timer.Elapsed.TotalSeconds);
            }
        }

        private static void Validate<T>(EventProvider p, EventBuilder eb, string name,
            Action<string, T, EventOutType, int> scalar,
            Action<string, T[], EventOutType, int> array,
            T value)
        {
            eb.Reset(name);
            eb.AddUInt8("A", 65, EventOutType.String);
            scalar("scalar", value, EventOutType.Default, 0);
            array("a0", new T[0], EventOutType.Default, 0);
            array("a1", new T[1] { value }, EventOutType.Default, 0);
            array("a2", new T[2] { value, value }, EventOutType.Default, 0);
            eb.AddUInt8("A", 65, EventOutType.String);
            p.Write(eb);
        }

        private static void ValidateBeginCount<T>(EventProvider p, EventBuilder eb, string name,
            Action<string, T, EventOutType, int> scalar,
            Action<string, T, int, int, EventOutType, int> scalarBeginCount,
            Action<string, T[], EventOutType, int> array,
            T value)
        {
            eb.Reset(name);
            eb.AddUInt8("A", 65, EventOutType.String);
            scalar("scalar", value, EventOutType.Default, 0);
            array("a0", new T[0], EventOutType.Default, 0);
            array("a1", new T[1] { value }, EventOutType.Default, 0);
            array("a2", new T[2] { value, value }, EventOutType.Default, 0);
            scalarBeginCount("scalar1-0", value, 1, 0, EventOutType.Default, 0);
            scalarBeginCount("scalar0-2", value, 0, 2, EventOutType.Default, 0);
            scalarBeginCount("scalar1-2", value, 1, 2, EventOutType.Default, 0);
            eb.AddUInt8("A", 65, EventOutType.String);
            p.Write(eb);
        }
    }
}
