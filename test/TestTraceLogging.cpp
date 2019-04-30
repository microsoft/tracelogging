// Copyright (c) Microsoft Corporation. All rights reserved.
// Licensed under the MIT License.

#include <catch2/catch.hpp>
#include <tracelogging/TraceLoggingProvider.h>

TRACELOGGING_DEFINE_PROVIDER(g_testProvider, "MyTestProvider",
                             (0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0));

TEST_CASE("TraceLogging workflow builds", "[tracelogging]") {
  TraceLoggingRegister(g_testProvider);

  int x = 0;
  TraceLoggingWrite(g_testProvider, "MyEventName",
                    TraceLoggingValue(x, "XVal"));

  TraceLoggingUnregister(g_testProvider);

  REQUIRE(true == true);
}