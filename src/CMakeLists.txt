cmake_minimum_required(VERSION 3.7)

set(LTTNGH_IncludeDir ${PROJECT_SOURCE_DIR}/include/lttnghroot/)

# Add source to this project's executable.
add_library(lttngh LttngActivityHelpers.c LttngHelpers.c)
target_compile_features(lttngh PRIVATE c_std_99)
target_include_directories(lttngh PUBLIC ${LTTNGH_IncludeDir})
target_link_libraries(lttngh PUBLIC lttng-ust lttng-ust-tracepoint)

set_property(TARGET lttngh PROPERTY POSITION_INDEPENDENT_CODE ON)
set_property(TARGET lttngh PROPERTY VERSION "1.0")
set_property(TARGET lttngh PROPERTY SOVERSION 1)

set(TraceLogging_IncludeDir ${PROJECT_SOURCE_DIR}/include/traceloggingroot/)

add_library(tracelogging INTERFACE)
target_include_directories(tracelogging INTERFACE ${TraceLogging_IncludeDir})
target_link_libraries(tracelogging INTERFACE lttngh)

install(TARGETS lttngh DESTINATION "${CMAKE_INSTALL_LIBDIR}")
install(DIRECTORY ${LTTNGH_IncludeDir} DESTINATION "${CMAKE_INSTALL_INCLUDEDIR}")
install(DIRECTORY ${TraceLogging_IncludeDir} DESTINATION "${CMAKE_INSTALL_INCLUDEDIR}")
