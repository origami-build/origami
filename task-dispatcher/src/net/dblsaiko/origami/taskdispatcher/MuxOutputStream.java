package net.dblsaiko.origami.taskdispatcher;

import java.io.IOException;
import java.io.OutputStream;

public class MuxOutputStream extends OutputStream {
    private final InheritableThreadLocal<OutputStream> streams = new InheritableThreadLocal<>();
    private final OutputStream fallback;

    public MuxOutputStream(OutputStream fallback) {
        this.fallback = fallback;
    }

    public void setThreadOutputStream(OutputStream ostr) {
        this.streams.set(ostr);
    }

    public OutputStream getThreadOutputStream() {
        OutputStream outputStream = this.streams.get();
        return outputStream != null ? outputStream : this.fallback;
    }

    @Override
    public void write(int b) throws IOException {
        OutputStream os = this.getThreadOutputStream();
        if (os == null) return;
        os.write(b);
    }

    @Override
    public void write(byte[] b) throws IOException {
        OutputStream os = this.getThreadOutputStream();
        if (os == null) return;
        os.write(b);
    }

    @Override
    public void write(byte[] b, int off, int len) throws IOException {
        OutputStream os = this.getThreadOutputStream();
        if (os == null) return;
        os.write(b, off, len);
    }

    @Override
    public void flush() throws IOException {
        OutputStream os = this.getThreadOutputStream();
        if (os == null) return;
        os.flush();
    }

    @Override
    public void close() throws IOException {
        OutputStream os = this.getThreadOutputStream();
        if (os == null) return;
        os.close();
    }
}
