package net.dblsaiko.origami.taskdispatcher;

import java.io.BufferedOutputStream;
import java.io.IOException;
import java.io.OutputStream;
import java.util.concurrent.ExecutionException;

import net.dblsaiko.origami.taskdispatcher.protocol.IoError;
import net.dblsaiko.origami.taskdispatcher.protocol.Result;
import net.dblsaiko.origami.taskdispatcher.protocol.Unit;

public class WrappedOutputStream extends OutputStream {
    private final int id;
    private final CommController ci;

    private WrappedOutputStream(int id, CommController ci) {
        this.id = id;
        this.ci = ci;
    }

    public static BufferedOutputStream create(int id, CommController ci) {
        return new BufferedOutputStream(new WrappedOutputStream(id, ci));
    }

    @Override
    public void write(int b) throws IOException {
        // this is inefficient, that's why the buffered output stream above
        this.write(new byte[] { (byte) b }, 0, 1);
    }

    @Override
    public void write(byte[] b, int off, int len) throws IOException {
        int pos = 0;

        while (pos < len) {
            Result<Integer, IoError> res;

            try {
                byte[] arr = new byte[len - pos];
                System.arraycopy(b, off, arr, pos, len - pos);
                res = this.ci.write(this.id, arr).get();
            } catch (InterruptedException | ExecutionException e) {
                throw new IOException(e);
            }

            if (res instanceof Result.Ok<Integer, IoError> ok) {
                pos += ok.val();
            } else if (res instanceof Result.Err<Integer, IoError> err) {
                throw err.error().toException();
            }
        }
    }

    @Override
    public void close() throws IOException {
        Result<Unit, IoError> res;

        try {
            res = this.ci.close(this.id).get();
        } catch (InterruptedException | ExecutionException e) {
            throw new IOException(e);
        }

        if (res instanceof Result.Err<Unit, IoError> e) {
            throw e.error().toException();
        }
    }
}
