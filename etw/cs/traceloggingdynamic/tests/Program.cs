namespace TraceLoggingDynamicTest
{
    using Microsoft.TraceLoggingDynamic;
    using System;
    using System.Diagnostics;
    using Encoding = System.Text.Encoding;
    using Stopwatch = System.Diagnostics.Stopwatch;

    /// <summary>
    /// Like ASCIIEncoding but instead of checking for non-ASCII characters, it just
    /// ignores the top bits of the character. This is significantly faster than
    /// ASCIIEncoding and can be used to improve performance in cases where you know a
    /// string only contains ASCII characters.
    /// </summary>
    class UncheckedASCIIEncoding : Encoding
    {
        public UncheckedASCIIEncoding()
            : base(20127) { }

        #region Required implementation of Encoding abstract methods

        public override int GetMaxByteCount(int charCount)
        {
            return charCount;
        }

        public override int GetMaxCharCount(int byteCount)
        {
            return byteCount;
        }

        public override int GetByteCount(char[] chars, int charIndex, int charCount)
        {
            return charCount;
        }

        public override int GetCharCount(byte[] bytes, int byteIndex, int byteCount)
        {
            return byteCount;
        }

        public unsafe override int GetBytes(char[] chars, int charIndex, int charCount, byte[] bytes, int byteIndex)
        {
            ValidateArgs(chars, charIndex, charCount, bytes, byteIndex, "char", "byte");
            fixed (char* charPtr = chars)
            {
                fixed (byte* bytePtr = bytes)
                {
                    return this.GetBytes(charPtr + charIndex, charCount, bytePtr + byteIndex, bytes.Length - byteIndex);
                }
            }
        }

        public unsafe override int GetChars(byte[] bytes, int byteIndex, int byteCount, char[] chars, int charIndex)
        {
            ValidateArgs(bytes, byteIndex, byteCount, chars, charIndex, "byte", "char");
            fixed (byte* bytePtr = bytes)
            {
                fixed (char* charPtr = chars)
                {
                    return this.GetChars(bytePtr + byteIndex, byteCount, charPtr + charIndex, chars.Length - charIndex);
                }
            }
        }

        #endregion

        #region Required overrides (used by the required implementation)

        public override unsafe int GetBytes(char* charPtr, int charCount, byte* bytePtr, int byteCount)
        {
            if (byteCount < charCount)
            {
                throw new ArgumentOutOfRangeException(nameof(byteCount));
            }

            for (int i = 0; i < charCount; i += 1)
            {
                bytePtr[i] = unchecked((byte)(charPtr[i] & 0x7F));
            }

            return charCount;
        }

        public override unsafe int GetChars(byte* bytePtr, int byteCount, char* charPtr, int charCount)
        {
            if (charCount < byteCount)
            {
                throw new ArgumentOutOfRangeException(nameof(charCount));
            }

            for (int i = 0; i < byteCount; i += 1)
            {
                charPtr[i] = (char)bytePtr[i];
            }

            return byteCount;
        }

        #endregion

        #region Optional overrides (performance/functionality improvement)

        public override bool IsSingleByte => true;

        public override int GetByteCount(string chars)
        {
            if (chars == null)
            {
                throw new ArgumentNullException(nameof(chars));
            }

            return chars.Length;
        }

        public override unsafe int GetByteCount(char* charPtr, int charCount)
        {
            return charCount;
        }

        public override unsafe int GetCharCount(byte* bytePtr, int byteCount)
        {
            return byteCount;
        }

        public unsafe override int GetBytes(string chars, int charIndex, int charCount, byte[] bytes, int byteIndex)
        {
            if (chars == null || bytes == null)
            {
                throw new ArgumentNullException(chars == null ? nameof(chars) : nameof(bytes));
            }
            else if (charIndex < 0 || charCount < 0)
            {
                throw new ArgumentOutOfRangeException(charIndex < 0 ? nameof(charIndex) : nameof(charCount));
            }
            else if (chars.Length - charIndex < charCount)
            {
                throw new ArgumentOutOfRangeException(chars);
            }
            else if (byteIndex < 0 || byteIndex > bytes.Length)
            {
                throw new ArgumentOutOfRangeException(nameof(byteIndex));
            }

            fixed (char* charPtr = chars)
            {
                fixed (byte* bytePtr = bytes)
                {
                    return this.GetBytes(charPtr + charIndex, charCount, bytePtr + byteIndex, bytes.Length - byteIndex);
                }
            }
        }

        #endregion

        private static void ValidateArgs<A, B>(A[] a, int aIndex, int aCount, B[] b, int bIndex, string aName, string bName)
        {
            if (a == null || b == null)
            {
                throw new ArgumentNullException(a == null ? aName + "s" : bName + "s");
            }
            else if (aIndex < 0 || aCount < 0)
            {
                throw new ArgumentOutOfRangeException(aIndex < 0 ? aName + "Index" : aName + "Count");
            }
            else if (a.Length - aIndex < aCount)
            {
                throw new ArgumentOutOfRangeException(aName + "s");
            }
            else if (bIndex < 0 || bIndex > b.Length)
            {
                throw new ArgumentOutOfRangeException(bName + "Index");
            }
        }
    }

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

            var eb = new EventBuilder(new UncheckedASCIIEncoding());
            var rawFields = eb.GetRawFields();
            Debug.Assert(rawFields.Item1.Length == 0);
            Debug.Assert(rawFields.Item2.Length == 0);
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
                rawFields = eb.GetRawFields();
                Debug.Assert(rawFields.Item1.Length == 0);
                Debug.Assert(rawFields.Item2.Length == 0);

                eb.Reset("tag0xFEDC000", EventLevel.Verbose, 1, 0xFEDC000);
                p.Write(eb);
                rawFields = eb.GetRawFields();
                Debug.Assert(rawFields.Item1.Length == 0);
                Debug.Assert(rawFields.Item2.Length == 0);

                eb.Reset("tag0xFEDCBAF", EventLevel.Verbose, 1, 0xFEDCBAF);
                p.Write(eb);
                rawFields = eb.GetRawFields();
                Debug.Assert(rawFields.Item1.Length == 0);
                Debug.Assert(rawFields.Item2.Length == 0);

                eb.Reset("fieldtag");
                eb.AddUInt8("0xFE00000", 0, EventOutType.Default, 0xFE00000);
                eb.AddUInt8("0xFEDC000", 0, EventOutType.Default, 0xFEDC000);
                eb.AddUInt8("0xFEDCBAF", 0, EventOutType.Default, 0xFEDCBAF);
                p.Write(eb);
                rawFields = eb.GetRawFields();

                eb.Reset("outtypes");
                eb.AppendRawFields(rawFields);
                eb.AddUInt8("default100", 100);
                eb.AddUInt8("string65", 65, EventOutType.String);
                eb.AddUInt8("bool1", 1, EventOutType.Boolean);
                eb.AddUInt8("hex100", 100, EventOutType.Hex);
                p.Write(eb);

                eb.Reset("structs");
                eb.AddUInt8("start", 0);
                var struct1Pos = eb.AddStruct("struct1", 127, 0xFE00000); // 127 = placeholder value
                {
                    byte struct1FieldCount = 0;

                    eb.AddUInt8("nested1", 1);
                    struct1FieldCount += 1;

                    eb.AddStruct("struct2", 1);
                    {
                        eb.AddUInt8("nested2", 2);
                    }
                    struct1FieldCount += 1;

                    eb.SetStructFieldCount(struct1Pos, struct1FieldCount); // replace placeholder with actual value
                }
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
                eb.AddAnsiString("zstr8e-", "", Encoding.ASCII);
                eb.AddAnsiString("zstr8e-a", "a", Encoding.ASCII);
                eb.AddAnsiString("zstr8e-0", "\0", Encoding.ASCII);
                eb.AddAnsiString("zstr8e-a0", "a\0", Encoding.ASCII);
                eb.AddAnsiString("zstr8e-0a", "\0a", Encoding.ASCII);
                eb.AddAnsiString("zstr8e-a0a", "a\0a", Encoding.ASCII);
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
                ValidateBeginCount(p, eb, "UnicodeString", eb.AddUnicodeString, eb.AddUnicodeString, eb.AddUnicodeStringArray, "utf16");
                ValidateBeginCount(p, eb, "CountedString", eb.AddCountedString, eb.AddCountedString, eb.AddCountedStringArray, "utf16");
                ValidateBeginCount(p, eb, "AnsiString", eb.AddAnsiString, eb.AddAnsiString, eb.AddAnsiStringArray, utf8.GetBytes("utf8"));
                ValidateBeginCount(p, eb, "AnsiStringE",
                    (n, v, o, t) => eb.AddAnsiString(n, v, Encoding.UTF8, o, t),
                    (n, v, s, c, o, t) => eb.AddAnsiString(n, v, Encoding.UTF8, s, c, o, t),
                    (n, v, o, t) => eb.AddAnsiStringArray(n, v, Encoding.UTF8, o, t),
                    "utf8");
                ValidateBeginCount(p, eb, "CountedAnsiString", eb.AddCountedAnsiString, eb.AddCountedAnsiString, eb.AddCountedAnsiStringArray, utf8.GetBytes("utf8"));
                ValidateBeginCount(p, eb, "CountedAnsiStringE",
                    (n, v, o, t) => eb.AddCountedAnsiString(n, v, Encoding.UTF8, o, t),
                    (n, v, s, c, o, t) => eb.AddCountedAnsiString(n, v, Encoding.UTF8, s, c, o, t),
                    (n, v, o, t) => eb.AddCountedAnsiStringArray(n, v, Encoding.UTF8, o, t),
                    "utf8");
                ValidateBeginCount(p, eb, "CountedBinary", eb.AddCountedBinary, eb.AddCountedBinary, eb.AddCountedBinaryArray, utf8.GetBytes("0123"));

                var val = utf8.GetBytes("val");
                var N = p.IsEnabled() ? 1 : 50000000;
                var timer = Stopwatch.StartNew();
                for (int i = 0; i < N; i++)
                {
                    eb.Reset("Benchmark");
                    eb.AddCountedString("f1", "now is the time for all good men to come to the aid of their country");
                    eb.AddInt32("f2", i);
                    eb.AddInt32("f2", i + 2);
                    eb.AddInt32("f2", i + 4);
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
