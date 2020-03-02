cmake_minimum_required(VERSION 3.5)

# Add source to this project's executable.
add_executable(traceloggingTest CatchMain.cpp TestTraceLogging.cpp)
target_link_libraries(traceloggingTest PRIVATE tracelogging Catch2::Catch2)

include(CTest)
include(Catch)
catch_discover_tests(traceloggingTest)
