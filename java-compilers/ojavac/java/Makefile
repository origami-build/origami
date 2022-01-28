JAVAC = ../../../target/debug/ojavac
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
	cd $(SRCDIR) && $(JAR) -cf $(PWD)/$@ $(RELOBJS)

%.class: %.java
	$(JAVAC) --release $(JAVA_VERSION) --package-root $(SRCDIR) --write-makedeps $(patsubst %.class,%.d,$@) -I$(SRCDIR) $<

clean:
	$(RM) ojavac.jar
	$(RM) $(OBJS) $(DEPS)

.PHONY: clean

-include $(DEPS)