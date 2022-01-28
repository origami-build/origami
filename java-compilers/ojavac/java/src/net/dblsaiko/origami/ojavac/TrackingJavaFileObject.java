package net.dblsaiko.origami.ojavac;

import java.io.IOException;
import java.io.InputStream;
import java.io.OutputStream;
import java.io.Reader;
import java.io.Writer;

import javax.tools.JavaFileObject;

public class TrackingJavaFileObject extends JavaFileObjectDelegate {
    private final TrackingJavaFileManager fm;

    public TrackingJavaFileObject(JavaFileObject delegate, TrackingJavaFileManager fm) {
        super(delegate);
        this.fm = fm;
    }

    @Override
    public InputStream openInputStream() throws IOException {
        this.fm.trackInput(this);
        return super.openInputStream();
    }

    @Override
    public OutputStream openOutputStream() throws IOException {
        this.fm.trackOutput(this);
        return super.openOutputStream();
    }

    @Override
    public Reader openReader(boolean ignoreEncodingErrors) throws IOException {
        this.fm.trackInput(this);
        return super.openReader(ignoreEncodingErrors);
    }

    @Override
    public Writer openWriter() throws IOException {
        this.fm.trackOutput(this);
        return super.openWriter();
    }

    @Override
    public CharSequence getCharContent(boolean ignoreEncodingErrors) throws IOException {
        this.fm.trackInput(this);
        return super.getCharContent(ignoreEncodingErrors);
    }
}
