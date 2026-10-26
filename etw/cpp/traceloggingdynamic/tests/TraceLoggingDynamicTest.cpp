// Copyright (c) Microsoft Corporation.
// Licensed under the MIT license.

#include <cassert>
#include <cstring>
#include <vector>
#include "..\TraceLoggingDynamic.h"

int main()
{
    std::vector<wchar_t> value(65535);
    for (unsigned i = 0; i != value.size(); ++i)
    {
        value[i] = static_cast<wchar_t>(i);
    }

    UINT16 const counts[] = { 0, 1, 32767, 32768, 65535 };
    for (UINT16 count : counts)
    {
        std::vector<BYTE> buffer;
        tld::EventDataBuilder<std::vector<BYTE>> builder(buffer);
        builder.AddCountedString(value.data(), count);
        UINT32 const sentinel = 0x12345678;
        builder.AddValue(sentinel);

        unsigned const expectedBytes = (count > 32767 ? 32767 : count) * sizeof(wchar_t);
        assert(buffer.size() == 2 + expectedBytes + sizeof(sentinel));
        assert((static_cast<unsigned>(buffer[0]) | (buffer[1] << 8)) == expectedBytes);
        assert(std::memcmp(buffer.data() + 2, value.data(), expectedBytes) == 0);
        assert(std::memcmp(buffer.data() + 2 + expectedBytes, &sentinel, sizeof(sentinel)) == 0);
    }
}
