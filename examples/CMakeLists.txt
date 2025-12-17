
cmake_minimum_required(VERSION 3.16)
project(mwalib_c_examples C)

set(CMAKE_C_STANDARD 11)
set(CMAKE_C_STANDARD_REQUIRED ON)

if(NOT CMAKE_BUILD_TYPE)
  set(CMAKE_BUILD_TYPE Release CACHE STRING "Build type" FORCE)
endif()
set_property(CACHE CMAKE_BUILD_TYPE PROPERTY STRINGS
  "Debug;Release;RelWithDebInfo;MinSizeRel")

set(CMAKE_C_FLAGS_DEBUG   "${CMAKE_C_FLAGS_DEBUG} -g")
set(CMAKE_C_FLAGS_RELEASE "${CMAKE_C_FLAGS_RELEASE} -O3")

# Paths
set(MWALIB_INCLUDE_DIR "${CMAKE_CURRENT_SOURCE_DIR}/../include")
set(MWALIB_LIBRARY_DIR_DEBUG   "../../target/debug")
set(MWALIB_LIBRARY_DIR_RELEASE "../../target/release")

# Define imported library with per-config artifacts
add_library(mwalib SHARED IMPORTED GLOBAL) # or SHARED if you have .so/.dylib
set_target_properties(mwalib PROPERTIES
  IMPORTED_LOCATION_DEBUG   "${MWALIB_LIBRARY_DIR_DEBUG}/libmwalib.a"   # or .so/.dylib
  IMPORTED_LOCATION_RELEASE "${MWALIB_LIBRARY_DIR_RELEASE}/libmwalib.a" # 
  INTERFACE_INCLUDE_DIRECTORIES "${MWALIB_INCLUDE_DIR}"
)

# Common system libs (portable alternative uses Threads::Threads)
add_library(common_system_libs INTERFACE)
target_link_libraries(common_system_libs INTERFACE m pthread dl)

# Binaries
add_executable(mwalib-print-context              mwalib-print-context.c)
add_executable(mwalib-print-volt-context         mwalib-print-volt-context.c)
add_executable(mwalib-sum-all-hdus               mwalib-sum-all-hdus.c)
add_executable(mwalib-sum-vcs                    mwalib-sum-vcs.c)

# Link to imported mwalib target + system libs
target_link_libraries(mwalib-print-context      PRIVATE mwalib common_system_libs)
target_link_libraries(mwalib-print-volt-context PRIVATE mwalib common_system_libs)
target_link_libraries(mwalib-sum-all-hdus       PRIVATE mwalib common_system_libs)
target_link_libraries(mwalib-sum-vcs            PRIVATE mwalib common_system_libs)
