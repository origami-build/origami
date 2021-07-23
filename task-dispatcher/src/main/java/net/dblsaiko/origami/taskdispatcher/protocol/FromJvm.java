package net.dblsaiko.origami.taskdispatcher.protocol;

import java.io.IOException;

public sealed interface FromJvm extends BinSerialize permits FromJvm.ExecResult, FromJvm.Read, FromJvm.WaitResult, FromJvm.Write, FromJvm.Close {
    final record ExecResult(net.dblsaiko.origami.taskdispatcher.protocol.ExecResult inner) implements FromJvm {
    }

    final record Write(net.dblsaiko.origami.taskdispatcher.protocol.Write inner) implements FromJvm {
    }

    final record Read(net.dblsaiko.origami.taskdispatcher.protocol.Read inner) implements FromJvm {
    }

    final record WaitResult(net.dblsaiko.origami.taskdispatcher.protocol.WaitResult inner) implements FromJvm {
    }

    final record Close(net.dblsaiko.origami.taskdispatcher.protocol.Close inner) implements FromJvm {
    }

    @Override
    default void serialize(BinSerializer serializer) throws IOException {
        if (this instanceof ExecResult execResult) {
            serializer.writeUsize(0);
            execResult.inner.serialize(serializer);
        } else if (this instanceof Write write) {
            serializer.writeUsize(1);
            write.inner.serialize(serializer);
        } else if (this instanceof Read read) {
            serializer.writeUsize(2);
            read.inner.serialize(serializer);
        } else if (this instanceof WaitResult waitResult) {
            serializer.writeUsize(3);
            waitResult.inner.serialize(serializer);
        } else if (this instanceof Close close) {
            serializer.writeUsize(4);
            close.inner.serialize(serializer);
        } else {
            throw new IllegalStateException("not implemented");
        }
    }
}
