.PHONY: all clean test

# non-versioned include
VARS ?= vars.mk
-include $(VARS)
JASMINE ?= $(shell which jasmine)

all: test

test:
	$(JASMINE) $(TESTOPTS)

clean:
	@echo "all clean..."

