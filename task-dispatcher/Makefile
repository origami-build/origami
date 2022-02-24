# if you don't have ojavac yet, set this to 1!
BOOTSTRAP ?= 0

JAVAC = ojavac
JAVAC_BOOTSTRAP = javac
JAR = jar

CLASSES := \
	AsyncQueue \
	CommController \
	Dispatcher \
	IncomingHandler \
	Main \
	MuxInputStream \
	MuxOutputStream \
	ProtocolReader \
	ProtocolWriter \
	Sink \
	Stream \
	Util \
	WrappedInputStream \
	WrappedOutputStream

EXTRA_CLASSES := \
	Util\$$IoRunnable

PROTO_CLASSES := \
	BinDeserializeFor \
	BinDeserializer \
	BinSerialize \
	BinSerializeFor \
	BinSerialize \
	Close \
	CloseResult \
	Exec \
	ExecError \
	ExecResult \
	FromJvm \
	IoError \
	Read \
	ReadResult \
	Result \
	Stdio \
	TaskInfo \
	ToJvm \
	Unit \
	Wait \
	WaitResult \
	Write \
	WriteResult

EXTRA_PROTO_CLASSES := \
	FromJvm\$$Close \
	ToJvm\$$CloseResult \
	IoError\$$1 \
	FromJvm\$$Read \
	FromJvm\$$WaitResult \
	Result\$$Err \
	ExecError\$$NoMainFn \
	ToJvm\$$Exec \
	Result\$$Ok \
	FromJvm\$$Write \
	ToJvm\$$ReadResult \
	ToJvm\$$Wait \
	ExecError\$$Failure \
	ExecError\$$InvalidClass \
	IoError\$$Kind \
	ToJvm\$$WriteResult \
	FromJvm\$$ExecResult

TEST_CLASSES := TestService

SRCDIR := src
PACKAGE := net/dblsaiko/origami/taskdispatcher
PROTO_PACKAGE := net/dblsaiko/origami/taskdispatcher/protocol
TEST_PACKAGE := net/dblsaiko/origami
JAVA_VERSION := 17

RELOBJS = \
	$(addprefix $(PACKAGE)/, $(addsuffix .class, $(CLASSES))) \
	$(addprefix $(PROTO_PACKAGE)/, $(addsuffix .class, $(PROTO_CLASSES))) \
	$(addprefix $(TEST_PACKAGE)/, $(addsuffix .class, $(TEST_CLASSES)))
OBJS = $(addprefix $(SRCDIR)/, $(RELOBJS))
DEPS = $(OBJS:%.class=%.d)

EXTRA_RELOBJS = $(RELOBJS) \
	$(addprefix $(PACKAGE)/, $(addsuffix .class, $(EXTRA_CLASSES))) \
	$(addprefix $(PROTO_PACKAGE)/, $(addsuffix .class, $(EXTRA_PROTO_CLASSES)))
EXTRA_OBJS = $(addprefix $(SRCDIR)/, $(EXTRA_RELOBJS))

all: task-dispatcher.jar

task-dispatcher.jar: $(OBJS)
	cd $(SRCDIR) && $(JAR) -cf $(CURDIR)/$@ $(EXTRA_RELOBJS)

%.class: %.java
ifeq ($(BOOTSTRAP), 0)
	$(JAVAC) --release $(JAVA_VERSION) --package-root $(SRCDIR) --write-makedeps $(patsubst %.class,%.d,$@) -I$(SRCDIR) $<
else
	$(JAVAC_BOOTSTRAP) -implicit:none --release $(JAVA_VERSION) -sourcepath $(SRCDIR) $<
endif

clean:
	$(RM) task-dispatcher.jar
	$(RM) $(EXTRA_OBJS) $(DEPS)

.PHONY: clean

-include $(DEPS)

.SUFFIXES:
