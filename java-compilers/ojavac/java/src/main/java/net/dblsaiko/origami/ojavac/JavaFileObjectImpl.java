package net.dblsaiko.origami.ojavac;

import java.io.IOException;
import java.io.InputStream;
import java.nio.file.Files;
import java.nio.file.Path;

import javax.tools.SimpleJavaFileObject;

public class JavaFileObjectImpl extends SimpleJavaFileObject {
    private final Path file;
    private String fileContent = null;

    protected JavaFileObjectImpl(Path file) {
        super(file.toUri(), Kind.SOURCE);
        this.file = file;
    }

    @Override
    public InputStream openInputStream() throws IOException {
        return Files.newInputStream(this.file);
    }

    @Override
    public CharSequence getCharContent(boolean ignoreEncodingErrors) throws IOException {
        if (this.fileContent == null) {
            this.fileContent = Files.readString(this.file);
        }

        return this.fileContent;
    }
}
