# if you don't have ojavac yet, set this to 1!
BOOTSTRAP ?= 0

JAVAC = ojavac
JAVAC_BOOTSTRAP = javac
JAR = jar

CLASSES := \
	Main \
	JavaFileObjectDelegate \
	StandardJavaFileManagerDelegate \
	TrackingJavaFileManager \
	TrackingJavaFileObject

SRCDIR := src
PACKAGE := net/dblsaiko/origami/ojavac
JAVA_VERSION := 16

RELOBJS = $(addprefix $(PACKAGE)/, $(addsuffix .class, $(CLASSES)))
OBJS = $(addprefix $(SRCDIR)/, $(RELOBJS))
DEPS = $(OBJS:%.class=%.d)

all: ojavac.jar

ojavac.jar: $(OBJS)
	cd $(SRCDIR) && $(JAR) -cf $(CURDIR)/$@ $(RELOBJS)

%.class: %.java
ifeq ($(BOOTSTRAP), 0)
	$(JAVAC) --release $(JAVA_VERSION) --package-root $(SRCDIR) --write-makedeps $(patsubst %.class,%.d,$@) -I$(SRCDIR) $<
else
	$(JAVAC_BOOTSTRAP) -implicit:none --release $(JAVA_VERSION) -sourcepath $(SRCDIR) $<
endif

clean:
	$(RM) ojavac.jar
	$(RM) $(OBJS) $(DEPS)

.PHONY: clean

-include $(DEPS)

.SUFFIXES: