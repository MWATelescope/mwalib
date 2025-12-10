
# ============================================================================
# Portable Makefile (GCC / Clang / MSVC) for mwalib examples  -- C11
# ============================================================================

# Use a visible recipe prefix instead of TABs (GNU Make ≥ 4.0)
.RECIPEPREFIX := >

# Choose compiler (override with: make CC=clang | make CC=gcc | make CC=cl)
CC ?= cc

# Detect compiler family: 'posix' (gcc/clang) or 'msvc' (cl)
COMPILER_FAMILY := posix
ifeq ($(notdir $(CC)),cl)
  COMPILER_FAMILY := msvc
endif

# Paths
INCLUDES := ../include
LIBDIR   := ../target/release

# Programs & sources
PROGS := mwalib-print-context \
         mwalib-sum-all-hdus \
         mwalib-print-volt-context \
         mwalib-sum-vcs

SRCS := $(addsuffix .c,$(PROGS))

# Flags (filled per compiler below)
CFLAGS  :=
LDFLAGS :=
LDLIBS  :=

# ---- POSIX toolchains: GCC / Clang -----------------------------------------
ifeq ($(COMPILER_FAMILY),posix)
  CFLAGS  += -std=c11 -Wall -Wextra -O3 -I $(INCLUDES)
  LDFLAGS += -L $(LIBDIR)
  LDLIBS  += -lm -lpthread -ldl -lmwalib
endif

# ---- MSVC (cl) --------------------------------------------------------------
ifeq ($(COMPILER_FAMILY),msvc)
  CFLAGS  += /std:c11 /W4 /O2 /I $(INCLUDES)
  # Ensure mwalib.lib exists in $(LIBDIR); you can also add "/link /LIBPATH:$(LIBDIR)"
  LDLIBS  += $(LIBDIR)/mwalib.lib
endif

# Default target
.PHONY: all
all: $(PROGS)

# Build rules
# POSIX rule (GCC/Clang)
ifeq ($(COMPILER_FAMILY),posix)
%: %.c
> $(CC) $(CFLAGS) $< -o $@ $(LDFLAGS) $(LDLIBS)
endif

# MSVC rule (cl compiles+links in one step)
ifeq ($(COMPILER_FAMILY),msvc)
%: %.c
> $(CC) $(CFLAGS) $< /Fe$@ $(LDLIBS)
endif

# Convenience run targets (POSIX shells)
.PHONY: run-mwalib-print-context run-mwalib-sum-all-hdus run-mwalib-print-volt-context run-mwalib-sum-vcs
run-mwalib-print-context: mwalib-print-context
> ./$<
run-mwalib-sum-all-hdus: mwalib-sum-all-hdus
> ./$<
run-mwalib-print-volt-context: mwalib-print-volt-context
> ./$<
run-mwalib-sum-vcs: mwalib-sum-vcs
> ./$<

# Cleaning
.PHONY: clean
clean:
> $(RM) $(PROGS) *.obj *.pdb *.ilk

# Debug helper
.PHONY: print-config
print-config:
> @echo "CC               = $(CC)"
> @echo "COMPILER_FAMILY  = $(COMPILER_FAMILY)"
> @echo "CFLAGS           = $(CFLAGS)"
> @echo "LDFLAGS          = $(LDFLAGS)"
> @echo "LDLIBS           = $(LDLIBS)"
> @echo "INCLUDES         = $(INCLUDES)"
> @echo "LIBDIR           = $(LIBDIR)"
