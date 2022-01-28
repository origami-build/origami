package net.dblsaiko.origami.ojavac;

import java.io.File;
import java.io.IOException;
import java.nio.file.Path;
import java.util.Collection;
import java.util.Iterator;
import java.util.ServiceLoader;
import java.util.Set;

import javax.tools.FileObject;
import javax.tools.JavaFileObject;
import javax.tools.JavaFileObject.Kind;
import javax.tools.StandardJavaFileManager;

public class StandardJavaFileManagerDelegate implements StandardJavaFileManager {
    protected final StandardJavaFileManager delegate;

    protected StandardJavaFileManagerDelegate(StandardJavaFileManager delegate) {
        this.delegate = delegate;
    }

    @Override
    public Iterable<? extends JavaFileObject> getJavaFileObjectsFromPaths(Collection<? extends Path> paths) {
        return this.delegate.getJavaFileObjectsFromPaths(paths);
    }

    @Override
    public Iterable<? extends JavaFileObject> getJavaFileObjects(Path... paths) {
        return this.delegate.getJavaFileObjects(paths);
    }

    @Override
    public void setLocationFromPaths(Location location, Collection<? extends Path> paths) throws IOException {
        this.delegate.setLocationFromPaths(location, paths);
    }

    @Override
    public void setLocationForModule(Location location, String moduleName, Collection<? extends Path> paths) throws IOException {
        this.delegate.setLocationForModule(location, moduleName, paths);
    }

    @Override
    public Iterable<? extends Path> getLocationAsPaths(Location location) {
        return this.delegate.getLocationAsPaths(location);
    }

    @Override
    public Path asPath(FileObject file) {
        return this.delegate.asPath(file);
    }

    @Override
    public void setPathFactory(PathFactory f) {
        this.delegate.setPathFactory(f);
    }

    @Override
    public Location getLocationForModule(Location location, String moduleName) throws IOException {
        return this.delegate.getLocationForModule(location, moduleName);
    }

    @Override
    public Location getLocationForModule(Location location, JavaFileObject fo) throws IOException {
        return this.delegate.getLocationForModule(location, fo);
    }

    @Override
    public <S> ServiceLoader<S> getServiceLoader(Location location, Class<S> service) throws IOException {
        return this.delegate.getServiceLoader(location, service);
    }

    @Override
    public String inferModuleName(Location location) throws IOException {
        return this.delegate.inferModuleName(location);
    }

    @Override
    public Iterable<Set<Location>> listLocationsForModules(Location location) throws IOException {
        return this.delegate.listLocationsForModules(location);
    }

    @Override
    public boolean contains(Location location, FileObject fo) throws IOException {
        return this.delegate.contains(location, fo);
    }

    @Override
    public ClassLoader getClassLoader(Location location) {
        return this.delegate.getClassLoader(location);
    }

    @Override
    public Iterable<JavaFileObject> list(Location location, String packageName, Set<Kind> kinds, boolean recurse) throws IOException {
        return this.delegate.list(location, packageName, kinds, recurse);
    }

    @Override
    public String inferBinaryName(Location location, JavaFileObject file) {
        return this.delegate.inferBinaryName(location, file);
    }

    @Override
    public boolean isSameFile(FileObject a, FileObject b) {
        return this.delegate.isSameFile(a, b);
    }

    @Override
    public boolean handleOption(String current, Iterator<String> remaining) {
        return this.delegate.handleOption(current, remaining);
    }

    @Override
    public boolean hasLocation(Location location) {
        return this.delegate.hasLocation(location);
    }

    @Override
    public JavaFileObject getJavaFileForInput(Location location, String className, Kind kind) throws IOException {
        return this.delegate.getJavaFileForInput(location, className, kind);
    }

    @Override
    public JavaFileObject getJavaFileForOutput(Location location, String className, Kind kind, FileObject sibling) throws IOException {
        return this.delegate.getJavaFileForOutput(location, className, kind, sibling);
    }

    @Override
    public FileObject getFileForInput(Location location, String packageName, String relativeName) throws IOException {
        return this.delegate.getFileForInput(location, packageName, relativeName);
    }

    @Override
    public FileObject getFileForOutput(Location location, String packageName, String relativeName, FileObject sibling) throws IOException {
        return this.delegate.getFileForOutput(location, packageName, relativeName, sibling);
    }

    @Override
    public void flush() throws IOException {
        this.delegate.flush();
    }

    @Override
    public void close() throws IOException {
        this.delegate.close();
    }

    @Override
    public Iterable<? extends JavaFileObject> getJavaFileObjectsFromFiles(Iterable<? extends File> files) {
        return this.delegate.getJavaFileObjectsFromFiles(files);
    }

    @Override
    public Iterable<? extends JavaFileObject> getJavaFileObjects(File... files) {
        return this.delegate.getJavaFileObjects(files);
    }

    @Override
    public Iterable<? extends JavaFileObject> getJavaFileObjectsFromStrings(Iterable<String> names) {
        return this.delegate.getJavaFileObjectsFromStrings(names);
    }

    @Override
    public Iterable<? extends JavaFileObject> getJavaFileObjects(String... names) {
        return this.delegate.getJavaFileObjects(names);
    }

    @Override
    public void setLocation(Location location, Iterable<? extends File> files) throws IOException {
        this.delegate.setLocation(location, files);
    }

    @Override
    public Iterable<? extends File> getLocation(Location location) {
        return this.delegate.getLocation(location);
    }

    @Override
    public int isSupportedOption(String option) {
        return this.delegate.isSupportedOption(option);
    }
}
