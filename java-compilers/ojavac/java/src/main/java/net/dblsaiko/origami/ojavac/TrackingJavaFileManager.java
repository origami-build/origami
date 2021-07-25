package net.dblsaiko.origami.ojavac;

import java.io.IOException;
import java.net.URI;
import java.net.URISyntaxException;
import java.nio.file.Path;
import java.util.Collection;
import java.util.Set;
import java.util.TreeSet;
import java.util.stream.StreamSupport;

import javax.tools.FileObject;
import javax.tools.JavaFileObject;
import javax.tools.JavaFileObject.Kind;
import javax.tools.StandardJavaFileManager;

public class TrackingJavaFileManager extends StandardJavaFileManagerDelegate {
    private final TreeSet<Path> inputFiles = new TreeSet<>();
    private final TreeSet<Path> outputFiles = new TreeSet<>();

    public TrackingJavaFileManager(StandardJavaFileManager delegate) {
        super(delegate);
    }

    @Override
    public JavaFileObject getJavaFileForInput(Location location, String className, Kind kind) throws IOException {
        JavaFileObject fo = super.getJavaFileForInput(location, className, kind);
        this.trackInput(fo);
        return fo;
    }

    @Override
    public JavaFileObject getJavaFileForOutput(Location location, String className, Kind kind, FileObject sibling) throws IOException {
        JavaFileObject fo = super.getJavaFileForOutput(location, className, kind, sibling);
        this.trackOutput(fo);
        return fo;
    }

    @Override
    public FileObject getFileForInput(Location location, String packageName, String relativeName) throws IOException {
        FileObject fo = super.getFileForInput(location, packageName, relativeName);
        this.trackInput(fo);
        return fo;
    }

    @Override
    public FileObject getFileForOutput(Location location, String packageName, String relativeName, FileObject sibling) throws IOException {
        FileObject fo = super.getFileForOutput(location, packageName, relativeName, sibling);
        this.trackOutput(fo);
        return fo;
    }

    @Override
    public Iterable<JavaFileObject> list(Location location, String packageName, Set<Kind> kinds, boolean recurse) throws IOException {
        Iterable<JavaFileObject> list = super.list(location, packageName, kinds, recurse);
        return () -> StreamSupport.stream(list.spliterator(), false).map(el -> (JavaFileObject) new TrackingJavaFileObject(el, this)).iterator();
    }

    @Override
    public Path asPath(FileObject file) {
        if (file instanceof JavaFileObjectDelegate delegate) {
            // unwrap because JavacFileManager uses instanceof checks on these
            // >_>
            return super.asPath(delegate.getDelegate());
        } else {
            return super.asPath(file);
        }
    }

    @Override
    public String inferBinaryName(Location location, JavaFileObject file) {
        if (file instanceof JavaFileObjectDelegate delegate) {
            // unwrap because JavacFileManager uses instanceof checks on these
            // >_>
            return super.inferBinaryName(location, delegate.getDelegate());
        } else {
            return super.inferBinaryName(location, file);
        }
    }

    public void trackInput(FileObject fo) {
        this.track(fo, this.inputFiles);
    }

    public void trackOutput(FileObject fo) {
        this.track(fo, this.outputFiles);
    }

    private void track(FileObject fo, Collection<Path> c) {
        if (fo == null) {
            return;
        }

        Path realPath = this.getRealPath(fo);

        if (realPath == null) {
            return;
        }

        c.add(realPath);
    }

    private Path getRealPath(FileObject fo) {
        URI uri = fo.toUri();

        return switch (uri.getScheme()) {
            case "file" -> Path.of(uri);
            // ignore jrt, we can't figure out the real path of the file anyway
            case "jrt" -> null;
            case "jar" -> {
                String schemeSpecificPart = uri.getSchemeSpecificPart();

                try {
                    yield Path.of(new URI(schemeSpecificPart.substring(0, schemeSpecificPart.indexOf('!'))));
                } catch (URISyntaxException e) {
                    // shouldn't happen because we're getting this from a valid
                    // URI that comes from Java code... but who knows
                    throw new RuntimeException(e);
                }
            }
            default -> throw new IllegalArgumentException("Don't know how to turn %s into a real path".formatted(uri));
        };
    }

    public TreeSet<Path> getInputFiles() {
        return this.inputFiles;
    }

    public TreeSet<Path> getOutputFiles() {
        return this.outputFiles;
    }
}
