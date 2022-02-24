package net.dblsaiko.origami.taskdispatcher.protocol;

import java.io.EOFException;
import java.io.FileNotFoundException;
import java.io.IOException;
import java.io.InterruptedIOException;
import java.net.ConnectException;
import java.net.SocketException;
import java.net.SocketTimeoutException;

public record IoError(Kind kind, String message) implements BinSerialize {
    public enum Kind {
        OTHER,
        NOT_FOUND,
        PERMISSION_DENIED,
        CONNECTION_REFUSED,
        CONNECTION_RESET,
        CONNECTION_ABORTED,
        NOT_CONNECTED,
        ADDR_IN_USE,
        ADDR_NOT_AVAILABLE,
        BROKEN_PIPE,
        ALREADY_EXISTS,
        WOULD_BLOCK,
        INVALID_INPUT,
        INVALID_DATA,
        TIMED_OUT,
        WRITE_ZERO,
        INTERRUPTED,
        UNEXPECTED_EOF,
    }

    @Override
    public void serialize(BinSerializer serializer) throws IOException {
        serializer.writeUsize(this.kind.ordinal());
        serializer.writeString(this.message);
    }

    public static IoError deserialize(BinDeserializer deserializer) throws IOException {
        var kind = deserializer.readUsize();
        var message = deserializer.readString();
        return new IoError(Kind.values()[kind], message);
    }

    static IoError fromException(IOException e) {
        Kind k;
        String message = e.getMessage();

        if (e instanceof FileNotFoundException) {
            k = Kind.NOT_FOUND;
        } else if (e instanceof ConnectException) {
            k = Kind.CONNECTION_REFUSED;
        } else if (e instanceof SocketTimeoutException) {
            k = Kind.TIMED_OUT;
        } else if (e instanceof InterruptedIOException) {
            k = Kind.INTERRUPTED;
        } else if (e instanceof EOFException) {
            k = Kind.UNEXPECTED_EOF;
        } else {
            k = Kind.OTHER;

            if (e.getClass() != IOException.class) {
                message = "%s: %s".formatted(e.getClass().getSimpleName(), e.getMessage());
            }
        }

        return new IoError(k, message);
    }

    public IOException toException() {
        return switch (this.kind) {
            case NOT_FOUND -> new FileNotFoundException(this.message);
            case CONNECTION_REFUSED -> new ConnectException(this.message);
            case CONNECTION_RESET -> new SocketException(this.message);
            case TIMED_OUT -> new SocketTimeoutException(this.message);
            case INTERRUPTED -> new InterruptedIOException(this.message);
            case UNEXPECTED_EOF -> new EOFException(this.message);
            default -> new IOException(this.message);
        };
    }
}
