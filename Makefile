prefix = /usr/local
exec_prefix = $(prefix)
bindir = $(exec_prefix)/bin
libexecdir = $(exec_prefix)/libexec

DEBUG = 0

CARGO ?= cargo
CARGO_OPTS += --target-dir make-target

INSTALL ?= install

ifneq ($(DEBUG),0)
CARGO_OUTDIR = make-target/debug
else
CARGO_OPTS += --release
CARGO_OUTDIR = make-target/release
endif

TARGETS = ojavac oresolve

.PHONY: all
all: cargo ojavac-java

.PHONY: install
install:
	$(INSTALL) -d $(DESTDIR)$(bindir) $(DESTDIR)$(libexecdir) || true
	$(INSTALL) -m755 $(addprefix $(CARGO_OUTDIR)/,$(TARGETS)) $(DESTDIR)$(bindir)
	$(INSTALL) java-compilers/ojavac/java/ojavac.jar $(DESTDIR)$(libexecdir)
	$(INSTALL) task-dispatcher/task-dispatcher.jar $(DESTDIR)$(libexecdir)

.PHONY: uninstall
uninstall:
	$(RM) $(addprefix $(DESTDIR)$(bindir)/,$(TARGETS))
	$(RM) $(DESTDIR)$(libexecdir)/ojavac.jar
	$(RM) $(DESTDIR)$(libexecdir)/task-dispatcher.jar

.PHONY: clean
clean:
	$(RM) -r make-target
	$(MAKE) -C java-compilers/ojavac/java BOOTSTRAP=1 clean

.PHONY: cargo
cargo:
	RUSTFLAGS='--cfg install' ORI_BUILD_LIBEXECDIR=$(libexecdir) $(CARGO) build $(CARGO_OPTS) $(addprefix --bin ,$(TARGETS))

.PHONY: ojavac-java
ojavac-java:
	$(MAKE) -C java-compilers/ojavac/java BOOTSTRAP=1 ojavac.jar

.PHONY: task-dispatcher
task-dispatcher:
	$(MAKE) -C task-dispatcher BOOTSTRAP=1 task-dispatcher.jar

.SUFFIXES: