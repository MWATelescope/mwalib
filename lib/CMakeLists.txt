#CMakeLists.txt
cmake_minimum_required(VERSION 3.5)
project(mwalib)

add_definitions(-D_GNU_SOURCE=1)

set(CMAKE_RUNTIME_OUTPUT_DIRECTORY ${CMAKE_HOME_DIRECTORY}/lib)

if(NOT CMAKE_BUILD_TYPE)
  set(CMAKE_BUILD_TYPE Release)
endif()

# define sources
set(PROGSRC src/args.c src/fitsreader.c src/gpubox.c)

IF(CMAKE_COMPILER_IS_GNUCXX)
    set(CMAKE_CXX_FLAGS_DEBUG "-g")
    set(CMAKE_CXX_FLAGS_RELEASE "-O3")
    SET(CMAKE_C_FLAGS "${CMAKE_C_FLAGS} -Wall -Werror -Wextra")
    SET(CMAKE_CXX_FLAGS "${CMAKE_CXX_FLAGS} -Wall -Werror -Wextra")
ENDIF(CMAKE_COMPILER_IS_GNUCXX)

if (USE_SANITIZER STREQUAL "Address")
    set(CMAKE_C_FLAGS "${CMAKE_C_FLAGS} -fno-omit-frame-pointer -fsanitize=address")
    set(CMAKE_CXX_FLAGS "${CMAKE_CXX_FLAGS} -fno-omit-frame-pointer -fsanitize=address")
elseif (USE_SANITIZER STREQUAL "Thread")
    set(CMAKE_C_FLAGS "${CMAKE_C_FLAGS} -fsanitize=thread")
    set(CMAKE_CXX_FLAGS "${CMAKE_CXX_FLAGS} -fsanitize=thread")
elseif (USE_SANITIZER STREQUAL "Undefined")
    set(CMAKE_C_FLAGS "${CMAKE_C_FLAGS} -fsanitize=undefined")
    set(CMAKE_CXX_FLAGS "${CMAKE_CXX_FLAGS} -fsanitize=undefined")
elseif (USE_SANITIZER STREQUAL "Leak")
    set(CMAKE_C_FLAGS "${CMAKE_C_FLAGS} -fsanitize=leak")
    set(CMAKE_CXX_FLAGS "${CMAKE_CXX_FLAGS} -fsanitize=leak")
elseif (USE_SANITIZER STREQUAL "Memory")
    set(CMAKE_C_FLAGS "${CMAKE_C_FLAGS} -fsanitize=memory")
    set(CMAKE_CXX_FLAGS "${CMAKE_CXX_FLAGS} -fsanitize=memory")
endif()

add_library(mwalib ${PROGSRC})       # define executable target prog, specify sources
target_include_directories(mwalib PUBLIC ${CMAKE_SOURCE_DIR})
# -l flags for linking target
if (USE_SANITIZER STREQUAL "Address")
    target_link_libraries(mwalib m pthread cfitsio -fsanitize=address)
elseif (USE_SANITIZER STREQUAL "Thread")
    target_link_libraries(mwalib m pthread cfitsio -fsanitize=thread)
elseif (USE_SANITIZER STREQUAL "Undefined")
    target_link_libraries(mwalib m pthread cfitsio -fsanitize=undefined)
elseif (USE_SANITIZER STREQUAL "Leak")
    target_link_libraries(mwalib m pthread cfitsio -fsanitize=leak)
elseif (USE_SANITIZER STREQUAL "Memory")
    target_link_libraries(mwalib m pthread cfitsio -fsanitize=address)
else ()
    target_link_libraries(mwalib m pthread cfitsio)
endif()
