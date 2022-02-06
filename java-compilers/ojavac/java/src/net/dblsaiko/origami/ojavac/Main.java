package net.dblsaiko.origami.ojavac;

import java.io.IOException;
import java.io.PrintWriter;
import java.nio.charset.StandardCharsets;
import java.nio.file.Files;
import java.nio.file.Path;
import java.nio.file.Paths;
import java.util.Arrays;

import javax.tools.JavaCompiler;
import javax.tools.JavaCompiler.CompilationTask;
import javax.tools.ToolProvider;

public class Main {
    public static void main(String[] args) {
        String classpath = args[args.length - 5];
        int optionsLen = Integer.parseInt(args[args.length - 4]);
        int compilationUnitsLen = Integer.parseInt(args[args.length - 3]);
        String manifestPath = args[args.length - 2];
        String makeManifestPath = args[args.length - 1];

        // the javac classes don't provide a defined API to set the classpath
        // pray this doesn't crash and burn when using a different jdk
        System.setProperty("env.class.path", classpath);

        JavaCompiler systemJavaCompiler = ToolProvider.getSystemJavaCompiler();
        TrackingJavaFileManager fm = new TrackingJavaFileManager(
            systemJavaCompiler.getStandardFileManager(null, null, StandardCharsets.UTF_8)
        );

        CompilationTask task = systemJavaCompiler.getTask(
            null,
            fm,
            null,
            iterSlice(args, 0, optionsLen),
            null,
            fm.getJavaFileObjectsFromPaths(Arrays.stream(args, optionsLen, optionsLen + compilationUnitsLen).map(Path::of).toList())
        );

        boolean result = task.call();

        if (!result) {
            System.exit(1);
        }

        if (!manifestPath.isBlank()) {
            writeManifest(iterSlice(args, optionsLen, optionsLen + compilationUnitsLen), fm, Path.of(manifestPath));
        }

        if (!makeManifestPath.isBlank()) {
            writeMakeManifest(iterSlice(args, optionsLen, optionsLen + compilationUnitsLen), fm, Path.of(makeManifestPath));
        }
    }

    private static <T> Iterable<T> iterSlice(T[] array, int start, int endExclusive) {
        return () -> Arrays.stream(array, start, endExclusive).iterator();
    }

    private static void writeManifest(Iterable<String> files, TrackingJavaFileManager fm, Path manifestPath) {
        try (PrintWriter pw = new PrintWriter(Files.newBufferedWriter(manifestPath))) {
            pw.print("; Dependencies for classes");

            for (String s : files) {
                pw.printf(" '%s'", s);
            }

            pw.println();
            pw.println("; Do not edit.");

            if (!fm.getInputFiles().isEmpty()) {
                pw.println();
                pw.println("; Inputs");

                for (Path inputFile : fm.getInputFiles()) {
                    pw.printf("<- %s\n", inputFile);
                }
            }

            if (!fm.getOutputFiles().isEmpty()) {
                pw.println();
                pw.println("; Outputs");

                for (Path outputFile : fm.getOutputFiles()) {
                    pw.printf("-> %s\n", outputFile);
                }
            }
        } catch (IOException e) {
            throw new RuntimeException(e);
        }
    }

    private static void writeMakeManifest(Iterable<String> files, TrackingJavaFileManager fm, Path manifestPath) {
        try (PrintWriter pw = new PrintWriter(Files.newBufferedWriter(manifestPath))) {
            pw.print("# Dependencies for classes");

            for (String s : files) {
                pw.printf(" '%s'", s);
            }

            pw.println();
            pw.println("# Do not edit.");

            Path pwd = Paths.get("").toAbsolutePath();

            if (!fm.getInputFiles().isEmpty()) {
                for (Path outputFile : fm.getOutputFiles()) {
                    pw.printf("%s:", stripPrefix(pwd, outputFile));

                    for (Path inputFile : fm.getInputFiles()) {
                        pw.printf(" %s", stripPrefix(pwd, inputFile));
                    }
                }
            }
        } catch (IOException e) {
            throw new RuntimeException(e);
        }
    }

    private static Path stripPrefix(Path parent, Path p) {
        if (p.startsWith(parent)) {
            return parent.relativize(p);
        }

        return p;
    }
}
