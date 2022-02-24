package net.dblsaiko.origami.taskdispatcher;

import java.io.EOFException;
import java.io.IOException;
import java.io.InputStream;
import java.io.OutputStream;

public class MuxInputStream extends InputStream {
    private final InheritableThreadLocal<InputStream> streams = new InheritableThreadLocal<>();
    private final InputStream fallback;

    public MuxInputStream(InputStream fallback) {
        this.fallback = fallback;
    }

    public void setThreadInputStream(InputStream ostr) {
        this.streams.set(ostr);
    }

    public InputStream getThreadInputStream() {
        InputStream inputStream = this.streams.get();
        return inputStream != null ? inputStream : this.fallback;
    }

    @Override
    public int read() throws IOException {
        InputStream is = this.getThreadInputStream();
        if (is == null) return -1;
        return is.read();
    }

    @Override
    public int read(byte[] b) throws IOException {
        InputStream is = this.getThreadInputStream();
        if (is == null) return -1;
        return is.read(b);
    }

    @Override
    public int read(byte[] b, int off, int len) throws IOException {
        InputStream is = this.getThreadInputStream();
        if (is == null) return -1;
        return is.read(b, off, len);
    }

    @Override
    public byte[] readAllBytes() throws IOException {
        InputStream is = this.getThreadInputStream();
        if (is == null) return new byte[0];
        return is.readAllBytes();
    }

    @Override
    public byte[] readNBytes(int len) throws IOException {
        InputStream is = this.getThreadInputStream();
        if (is == null) return new byte[0];
        return is.readNBytes(len);
    }

    @Override
    public int readNBytes(byte[] b, int off, int len) throws IOException {
        InputStream is = this.getThreadInputStream();
        if (is == null) return 0;
        return is.readNBytes(b, off, len);
    }

    @Override
    public long skip(long n) throws IOException {
        InputStream is = this.getThreadInputStream();
        if (is == null) return 0;
        return is.skip(n);
    }

    @Override
    public void skipNBytes(long n) throws IOException {
        InputStream is = this.getThreadInputStream();
        if (is == null) throw new EOFException("could not get thread stream");
        is.skipNBytes(n);
    }

    @Override
    public int available() throws IOException {
        InputStream is = this.getThreadInputStream();
        if (is == null) return 0;
        return is.available();
    }

    @Override
    public void close() throws IOException {
        InputStream is = this.getThreadInputStream();
        if (is == null) return;
        is.close();
    }

    @Override
    public synchronized void mark(int readlimit) {
        InputStream is = this.getThreadInputStream();
        if (is == null) return;
        is.mark(readlimit);
    }

    @Override
    public synchronized void reset() throws IOException {
        InputStream is = this.getThreadInputStream();
        if (is == null) throw new IOException("could not get thread stream");
        is.reset();
    }

    @Override
    public boolean markSupported() {
        InputStream is = this.getThreadInputStream();
        if (is == null) return false;
        return is.markSupported();
    }

    @Override
    public long transferTo(OutputStream out) throws IOException {
        InputStream is = this.getThreadInputStream();
        if (is == null) return 0;
        return is.transferTo(out);
    }
}
