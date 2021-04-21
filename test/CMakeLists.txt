cmake_minimum_required(VERSION 3.5)

add_executable(
    traceloggingTest
    CatchMain.cpp
    TestTraceLogging.cpp
    TestTraceLogging.c)

# FIXME: lttng-ust-common only for 2.13 and later.
# (Should be handled by the LTTngUST package?)
target_link_libraries(
    traceloggingTest
    PRIVATE
    lttng-ust-common
    tracelogging
    Catch2::Catch2)

include(CTest)
include(Catch)
catch_discover_tests(traceloggingTest)
