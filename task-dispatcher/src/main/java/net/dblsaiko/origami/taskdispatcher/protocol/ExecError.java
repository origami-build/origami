package net.dblsaiko.origami.taskdispatcher.protocol;

import java.io.IOException;

public sealed interface ExecError extends BinSerialize permits ExecError.Failure, ExecError.InvalidClass, ExecError.NoMainFn {
    final record Failure(String message) implements ExecError {
    }

    final record InvalidClass(String message) implements ExecError {
    }

    final record NoMainFn(String message) implements ExecError {
    }

    @Override
    default void serialize(BinSerializer serializer) throws IOException {
        if (this instanceof Failure failure) {
            serializer.writeUsize(0);
            serializer.writeString(failure.message());
        } else if (this instanceof InvalidClass invalidClass) {
            serializer.writeUsize(1);
            serializer.writeString(invalidClass.message());
        } else if (this instanceof NoMainFn noMainFn) {
            serializer.writeUsize(2);
            serializer.writeString(noMainFn.message());
        }
    }
}
