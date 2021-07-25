package net.dblsaiko.origami.ojavac;

import java.io.IOException;
import java.io.InputStream;
import java.io.OutputStream;
import java.io.Reader;
import java.io.Writer;
import java.net.URI;

import javax.lang.model.element.Modifier;
import javax.lang.model.element.NestingKind;
import javax.tools.JavaFileObject;

public class JavaFileObjectDelegate implements JavaFileObject {
    protected final JavaFileObject delegate;

    protected JavaFileObjectDelegate(JavaFileObject delegate) {
        this.delegate = delegate;
    }

    @Override
    public Kind getKind() {
        return this.delegate.getKind();
    }

    @Override
    public boolean isNameCompatible(String simpleName, Kind kind) {
        return this.delegate.isNameCompatible(simpleName, kind);
    }

    @Override
    public NestingKind getNestingKind() {
        return this.delegate.getNestingKind();
    }

    @Override
    public Modifier getAccessLevel() {
        return this.delegate.getAccessLevel();
    }

    @Override
    public URI toUri() {
        return this.delegate.toUri();
    }

    @Override
    public String getName() {
        return this.delegate.getName();
    }

    @Override
    public InputStream openInputStream() throws IOException {
        return this.delegate.openInputStream();
    }

    @Override
    public OutputStream openOutputStream() throws IOException {
        return this.delegate.openOutputStream();
    }

    @Override
    public Reader openReader(boolean ignoreEncodingErrors) throws IOException {
        return this.delegate.openReader(ignoreEncodingErrors);
    }

    @Override
    public CharSequence getCharContent(boolean ignoreEncodingErrors) throws IOException {
        return this.delegate.getCharContent(ignoreEncodingErrors);
    }

    @Override
    public Writer openWriter() throws IOException {
        return this.delegate.openWriter();
    }

    @Override
    public long getLastModified() {
        return this.delegate.getLastModified();
    }

    @Override
    public boolean delete() {
        return this.delegate.delete();
    }

    public JavaFileObject getDelegate() {
        return this.delegate;
    }
}
