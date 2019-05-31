cmake_minimum_required(VERSION 3.5)

set(TLG_INCLUDEDIR ${PROJECT_SOURCE_DIR}/include/)

# Add source to this project's executable.
add_library(lttngh LttngActivityHelpers.c LttngHelpers.c)

if (NOT CMAKE_VERSION VERSION_LESS 3.8.2)
    target_compile_features(lttngh PRIVATE c_std_99)
endif ()

target_include_directories(lttngh PUBLIC $<INSTALL_INTERFACE:include> $<BUILD_INTERFACE:${TLG_INCLUDEDIR}>)
target_link_libraries(lttngh PUBLIC lttng-ust lttng-ust-tracepoint)

set_property(TARGET lttngh PROPERTY POSITION_INDEPENDENT_CODE ON)
set_property(TARGET lttngh PROPERTY VERSION "1.0")
set_property(TARGET lttngh PROPERTY SOVERSION 1)

add_library(tracelogging INTERFACE)
target_include_directories(tracelogging INTERFACE $<INSTALL_INTERFACE:include> $<BUILD_INTERFACE:${TLG_INCLUDEDIR}>)
target_link_libraries(tracelogging INTERFACE lttngh)

set(TARGETLIST tracelogging lttngh)

install(DIRECTORY ${TLG_INCLUDEDIR} DESTINATION ${CMAKE_INSTALL_INCLUDEDIR})
install(TARGETS ${TARGETLIST} EXPORT tracelogging DESTINATION ${CMAKE_INSTALL_LIBDIR})

install(
    EXPORT tracelogging
    DESTINATION ${CMAKE_INSTALL_LIBDIR}/cmake/tracelogging
    FILE traceloggingConfig.cmake)
