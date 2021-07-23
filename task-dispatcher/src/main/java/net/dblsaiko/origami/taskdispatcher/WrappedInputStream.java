package net.dblsaiko.origami.taskdispatcher;

import java.io.BufferedInputStream;
import java.io.IOException;
import java.io.InputStream;
import java.util.concurrent.ExecutionException;

import net.dblsaiko.origami.taskdispatcher.protocol.IoError;
import net.dblsaiko.origami.taskdispatcher.protocol.Result;

public class WrappedInputStream extends InputStream {
    private final int id;
    private final CommController ci;

    private WrappedInputStream(int id, CommController ci) {
        this.id = id;
        this.ci = ci;
    }

    public static BufferedInputStream create(int id, CommController ci) {
        return new BufferedInputStream(new WrappedInputStream(id, ci));
    }

    @Override
    public int read() throws IOException {
        byte[] a = { 0 };

        if (this.read(a, 0, 1) == -1) {
            return -1;
        } else {
            return a[0];
        }
    }

    @Override
    public int read(byte[] b, int off, int len) throws IOException {
        Result<byte[], IoError> res;

        try {
            res = this.ci.read(this.id, len).get();
        } catch (InterruptedException | ExecutionException e) {
            throw new IOException(e);
        }

        if (res instanceof Result.Ok<byte[], IoError> ok) {
            System.arraycopy(ok.val(), 0, b, off, ok.val().length);
            return ok.val().length;
        } else if (res instanceof Result.Err<byte[], IoError> err) {
            throw err.error().toException();
        } else throw new IllegalStateException("unreachable");
    }
}
